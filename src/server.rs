use mlua::prelude::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::command::Command;
use crate::database::{Database, DatabaseProxy, Object, Verb, World};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use tokio::sync::mpsc;

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

fn format_error(e: &LuaError) -> String {
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

        let (read, write) = socket.into_split();

        let uuid = do_login_command(system_uuid, &lua);

        inject_notify_function(&lua, our_txs.clone());
        let (notify_tx, notify_rx) = create_notify_channel(our_txs, uuid);
        spawn_notify_task(notify_rx, write);

        let (line_tx, line_rx) = async_channel::unbounded::<String>();
        spawn_read_task(read, line_tx);
        inject_read_function(&lua, line_rx.clone());

        let (eval_tx, eval_rx) = mpsc::channel::<(String, String)>(100);
        spawn_processing_task(uuid, eval_tx, notify_tx, line_rx.clone(), db);

        tokio::spawn(async move {
            eval_task(lua, eval_rx, notify_tx.clone()).await;
        });
    }

    // TODO notify players when a player in the same room disconnects
}

async fn eval_task(
    lua: Lua,
    mut eval_rx: mpsc::Receiver<(String, String)>,
    notify_tx: mpsc::Sender<String>,
) {
    while let Some((chunk_name, lua_code)) = eval_rx.recv().await {
        println!("{}\n{}", chunk_name, lua_code);
        let chunk = lua.load(&lua_code).set_name(&chunk_name).unwrap();
        let future = chunk.eval_async();
        let lua_result = future.await;

        let msg = match lua_result {
            Err(e) => format_error(&e),
            Ok(LuaValue::String(s)) => s.to_str().unwrap().to_string(),
            Ok(v) => format!("{:?}", v),
        };

        if let Err(e) = notify_tx.send(msg).await {
            println!("spawn_eval_task -> notify_tx.send: {}", e);
        }
    }
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
    eval_tx: mpsc::Sender<(String, String)>,
    notify_tx: mpsc::Sender<String>,
    line_rx: async_channel::Receiver<String>,
    db: Arc<RwLock<Database>>,
) {
    tokio::spawn(async move {
        let conndata = ConnData {
            player_object: uuid,
        };
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
            CONNDATA
                .scope(conndata, async {
                    let res = if let Some(stripped) = line.strip_prefix(';') {
                        execute_player_lua_command(stripped)
                    } else {
                        execute_verb(db.clone(), line)
                    };
                    match res {
                        Ok(eval) => eval_tx.send(eval).await.ok(),
                        Err(s) => notify_tx.send(s).await.ok(),
                    }
                })
                .await;
        }
    });
}

fn execute_player_lua_command(stripped: &str) -> Result<(String, String), String> {
    Ok((";-command".to_string(), stripped.to_string()))
}

fn execute_verb(db: Arc<RwLock<Database>>, line: String) -> Result<(String, String), String> {
    // Otherwise, as a ROO command
    let player_uuid = &CONNDATA.get().player_object;
    let command_opt = {
        let lock = db.read().unwrap();
        Command::parse(&line, player_uuid, &lock)
    };

    let command = match command_opt {
        Some(cmd) => cmd,
        None => return Err("Failed to parse command".to_string()),
    };

    // Find where
    let lock = db.read().unwrap();
    let location: Option<&Object> = {
        lock.get(player_uuid)
            .unwrap()
            .location()
            .and_then(|l| lock.get(l).ok())
    };
    // Find what
    let (this, verb) = match first_matching_verb(
        &lock,
        &command,
        vec![Some(lock.get(player_uuid).unwrap()), location],
    ) {
        Ok(Some(x)) => x,
        Ok(None) => return Err("Unknown verb".to_string()),
        Err(s) => return Err(s),
    };

    let dobjstr = command.dobjstr();
    let argstr = command.argstr();
    let this_uuid = this.uuid().to_string();
    let verb_name = verb.names()[0].clone();
    let args = command
        .args()
        .iter()
        .map(|s| format!("{:?}", s))
        .collect::<Vec<_>>()
        .join(", ");

    Ok((
        format!("{}:{}({})", this.name(), verb_name, args),
        format!(
            "
            player = toobj({:?}):unwrap()
            dobjstr = {:?}
            argstr = {:?}
            return toobj({:?}):unwrap():{}({})
        ",
            player_uuid.to_string(),
            dobjstr,
            argstr,
            this_uuid,
            verb_name,
            args
        ),
    ))
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
