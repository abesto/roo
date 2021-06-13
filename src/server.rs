use mlua::prelude::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::command::Command;
use crate::database::{Database, DatabaseProxy, Object, Verb, World};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};
use tokio::sync::{mpsc, oneshot};

#[derive(Copy, Clone)]
pub struct ConnData {
    pub player_object: Uuid,
}

tokio::task_local! {
    pub static CONNDATA: ConnData
}

fn first_matching_verb<'a, 'b>(
    lock: &'b RwLockReadGuard<Database>,
    command: &'a Command,
    objects: Vec<Option<&'b Object>>,
) -> Result<Option<(&'b Object, &'b Verb)>, String> {
    for object_opt in objects {
        if let Some(object) = object_opt {
            if let Some(verb) = lock.matching_verb(object.uuid(), command)? {
                return Ok(Some((object, verb)));
            }
        }
    }
    Ok(None)
}

fn get_object_proxy<'lua>(lua: &'lua Lua, uuid: &Uuid) -> LuaValue<'lua> {
    lua.load(&format!("db[\"{}\"]", uuid.to_string()))
        .eval::<LuaValue>()
        .unwrap()
}

fn format_error(e: LuaError) -> String {
    match &e {
        LuaError::CallbackError { cause, .. } => {
            format!("{}\n{}", cause, e)
        }
        _ => e.to_string(),
    }
}

#[tokio::main]
pub async fn run_server(world: World) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8888").await?;
    let txs: Arc<RwLock<HashMap<Uuid, mpsc::Sender<String>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await?;
        let our_txs = txs.clone();
        let db = world.db();
        let system_uuid = db.read().unwrap().system_uuid().clone();
        let lua = CONNDATA.sync_scope(
            ConnData {
                player_object: system_uuid,
            },
            || world.lua(),
        );

        tokio::spawn(async move {
            let (read, write) = socket.into_split();

            let uuid = do_login_command(system_uuid, &lua);

            inject_notify_function(&lua, our_txs.clone());
            let (notify_tx, notify_rx) = create_notify_channel(our_txs, uuid);
            spawn_notify_task(notify_rx, write);

            let (line_tx, line_rx) = async_channel::unbounded::<String>();
            spawn_read_task(read, line_tx);
            inject_read_function(&lua, line_rx.clone());
            spawn_processing_task(uuid, notify_tx, line_rx.clone(), lua, db);
        });
    }

    // TODO notify players when a player in the same room disconnects
}

fn inject_read_function(lua: &Lua, line_rx: async_channel::Receiver<String>) {
    lua.globals()
        .set(
            "_server_read",
            lua.create_function(move |_lua, ()| {
                // TODO Rust currently doesn't support async move closures, so we cannot use
                // async line_rx.recv().await, so the injected function is non-blocking.
                // We need Lua-side polling / wait.
                match line_rx.try_recv() {
                    Ok(l) => Ok(Some(l)),
                    Err(async_channel::TryRecvError::Empty) => Ok(None),
                    Err(e) => Err(LuaError::external(e.to_string())),
                }
            })
            .unwrap(),
        )
        .unwrap();
}

fn spawn_read_task(read: OwnedReadHalf, line_tx: async_channel::Sender<String>) {
    let mut lines = BufReader::new(read).lines();
    tokio::spawn(async move {
        while let Some(line) = lines.next_line().await.unwrap() {
            line_tx.send(line).await.unwrap();
        }
    });
}

fn spawn_processing_task(
    uuid: Uuid,
    notify_tx: mpsc::Sender<String>,
    line_rx: async_channel::Receiver<String>,
    lua: Lua,
    db: Arc<RwLock<Database>>,
) {
    tokio::spawn(async move {
        let conndata = ConnData {
            player_object: uuid,
        };

        CONNDATA
            .scope(conndata, async move {
                notify_tx.send("Hai!".to_string()).await.unwrap();
                loop {
                    let line = match line_rx.recv().await {
                        Ok(l) => l,
                        Err(e) => {
                            // TODO err logging
                            println!("{}", e);
                            break;
                        }
                    };
                    println!("< {}", line);
                    let lua_arc = Arc::new(lua);
                    let maybe_msg = if let Some(stripped) = line.strip_prefix(';') {
                        execute_player_lua_command(lua_arc, stripped)
                    } else {
                        execute_verb(lua_arc, db, line).await
                    };
                    if let Some(msg) = maybe_msg {
                        notify_tx.send(msg.to_string()).await.unwrap();
                    }
                }
                Ok::<(), std::io::Error>(())
            })
            .await
            .unwrap();
    });
}

fn execute_player_lua_command(lua: Arc<Lua>, stripped: &str) -> Option<String> {
    match lua
        .load(stripped)
        .set_name(";-command")
        .unwrap()
        .eval::<LuaValue>()
    {
        Err(e) => Some(format_error(e)),
        Ok(LuaValue::String(s)) => Some(s.to_str().unwrap().to_string()),
        Ok(v) => Some(format!("{:?}", v)),
    }
}

async fn execute_verb(lua: Arc<Lua>, db: Arc<RwLock<Database>>, line: String) -> Option<String> {
    // Otherwise, as a ROO command
    let player_uuid = &CONNDATA.get().player_object;
    let command_opt = {
        let lock = db.read().unwrap();
        Command::parse(&line, player_uuid, &lock)
    };

    let command = match command_opt {
        Some(cmd) => cmd,
        None => return Some("Failed to parse command".to_string()),
    };

    let lua_code_res = {
        // Find where
        let lock = db.read().unwrap();
        let location: Option<&Object> = {
            lock.get(player_uuid)
                .unwrap()
                .location()
                .and_then(|l| lock.get(l).ok())
        };
        // Find what
        let (this, verb) = {
            first_matching_verb(
                &lock,
                &command,
                vec![Some(lock.get(player_uuid).unwrap()), location],
            )?
            .ok_or_else(|| "Unknown verb".to_string())?
        };

        // Set up arguments
        let lua_player = get_object_proxy(&lua, player_uuid);
        lua.globals().set("player", lua_player).unwrap();
        lua.globals().set("dobjstr", command.dobjstr()).unwrap();
        lua.globals()
            .set("argstr", command.argstr().clone())
            .unwrap();

        // Generate the Lua code to execute the requested verb
        Ok((
            format!(
                "coroutine.create(function(...) db[\"{}\"]:resolve_verb(\"{}\")(...) end)",
                this.uuid(),
                verb.names()[0],
            ),
            this.uuid().clone(),
        ))
    };

    let (lua_code, this_uuid) = match lua_code_res {
        Ok(s) => s,
        Err(s) => return Some(s),
    };
    let args = command.args().clone().to_lua(&lua).unwrap();
    let lua_this = get_object_proxy(&lua, &this_uuid);

    // Create the LuaThread to execute the command
    let thread_res = lua
        .load(&lua_code)
        .set_name(&lua_code)
        .unwrap()
        .eval::<LuaThread>()
        .map(|t| t.into_async::<_, LuaValue>((lua_this, args)))
        .map_err(format_error);

    match thread_res {
        Ok(thread) => match thread.await {
            Ok(_) => None,
            Err(e) => Some(format_error(e)),
        },
        Err(e) => Some(e),
    }
}

fn spawn_notify_task(
    mut notify_rx: mpsc::Receiver<String>,
    mut write: tokio::net::tcp::OwnedWriteHalf,
) {
    tokio::spawn(async move {
        while let Some(msg) = notify_rx.recv().await {
            let processed = {
                let a = msg.replace("\n", "\r\n");
                if !a.ends_with("\r\n") {
                    format!("{}\r\n", a)
                } else {
                    a
                }
            };
            print!("> {}", processed);
            write.write_all(processed.as_bytes()).await.unwrap();
        }
    });
}

fn create_notify_channel(
    our_txs: Arc<RwLock<HashMap<Uuid, mpsc::Sender<String>>>>,
    uuid: Uuid,
) -> (mpsc::Sender<String>, mpsc::Receiver<String>) {
    let (tx, rx) = mpsc::channel::<String>(100);
    {
        let mut lock = our_txs.write().unwrap();
        lock.insert(uuid, tx.clone());
    }
    (tx, rx)
}

fn do_login_command(system_uuid: Uuid, lua: &Lua) -> Uuid {
    let uuid = CONNDATA.sync_scope(
        ConnData {
            player_object: system_uuid,
        },
        || {
            let uuid_str = lua
                .load("system:do_login_command()")
                .set_name("system:do_login_command()")
                .unwrap()
                .eval::<String>()
                .unwrap();
            DatabaseProxy::parse_uuid(&uuid_str).unwrap()
        },
    );
    uuid
}

fn inject_notify_function(lua: &Lua, notify_txs: Arc<RwLock<HashMap<Uuid, mpsc::Sender<String>>>>) {
    lua.globals()
        .set(
            "_server_notify",
            lua.create_function(move |_lua, (uuid, msg): (String, String)| {
                let lock = notify_txs.read().unwrap();
                if let Some(tx) = lock.get(&DatabaseProxy::parse_uuid(&uuid)?) {
                    // TODO handle buffer full
                    return tx.try_send(msg).map_err(LuaError::external);
                }
                Ok(())
            })
            .unwrap(),
        )
        .unwrap();
}
