use mlua::prelude::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::command::Command;
use crate::database::{Database, DatabaseProxy, Object, Verb, World};
use crate::saveload::SaveloadConfig;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::time::Duration;
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
    for object in objects.into_iter().flatten() {
        if let Some(verb) = lock.matching_verb(object.uuid(), command)? {
            return Ok(Some((object, verb)));
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
pub async fn run_server(world: World, saveload_config: SaveloadConfig) {
    tokio::task::LocalSet::new()
        .run_until(server_main(world, saveload_config))
        .await
        .unwrap()
}

struct Cleanup {
    db: Arc<RwLock<Database>>,
    saveload_config: SaveloadConfig,
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        println!("Shutting down, writing final DB checkpoint");
        let lock = self.db.read().unwrap();
        self.saveload_config.checkpoint(&lock).unwrap();
    }
}

pub async fn server_main(
    mut world: World,
    saveload_config: SaveloadConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8888").await?;
    let notify_txs: Arc<RwLock<HashMap<Uuid, mpsc::Sender<String>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let db = world.db();

    let _cleanup = Cleanup {
        db: db.clone(),
        saveload_config: saveload_config.clone(),
    };
    spawn_checkpointing_task(db.clone(), saveload_config);

    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        shutdown_tx.send(()).unwrap();
    });

    println!("Server started");

    loop {
        tokio::select! {
                _ = &mut shutdown_rx => {
                    break Ok(())
                },
                result = listener.accept() => {
                    let (socket, _) = result?;
                    let not_logged_in_uuid = {
                        let mut db = db.write().unwrap();
                        db.create_orphan()
                    };

                    CONNDATA.scope(
                        ConnData {
                            player_object: not_logged_in_uuid,
                        },
                        async {
                            let lua = world.lua();
                    let (read, write) = socket.into_split();

                    let (notify_tx, notify_rx) = create_notify_channel(notify_txs.clone(), not_logged_in_uuid);
                    inject_notify_function(&lua, notify_txs.clone());
                    spawn_notify_task(notify_rx, write);

                    let (line_tx, line_rx) = async_channel::unbounded::<String>();
                    spawn_read_task(read, line_tx);
                    inject_read_function(&lua, line_rx.clone());

                    if let Some(uuid) = do_login_command(&lua, line_rx.clone(), notify_tx.clone(), db.clone()).await {
                        move_notify_channel(notify_txs.clone(), &not_logged_in_uuid, uuid);
                        CONNDATA.scope(
                            ConnData { player_object: uuid },
                            async {
                                spawn_processing_task(lua, uuid, notify_tx.clone(), line_rx.clone(), db.clone());
                            }
                        ).await;
                    }
                }).await;
            }
        }
    }

    // TODO notify players when a player in the same room disconnects
}

fn spawn_checkpointing_task(db: Arc<RwLock<Database>>, saveload_config: SaveloadConfig) {
    tokio::task::spawn_local(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(100)).await;
            println!("Starting periodic DB checkpointing");
            {
                let lock = db.read().unwrap();
                match saveload_config.checkpoint(&lock) {
                    Err(e) => println!("Checkpointing failed: {}", e),
                    Ok(f) => println!("Checkpointing done, saved to {}", f),
                }
            }
        }
    });
}

async fn eval(lua: &Lua, chunk_name: String, lua_code: String, notify_tx: &mpsc::Sender<String>) {
    println!("{}\n{}", chunk_name, lua_code);
    let chunk = lua.load(&lua_code).set_name(&chunk_name).unwrap();
    let future = chunk.eval_async();
    let lua_result = future.await;

    let msg = match lua_result {
        Err(e) => format_error(&e),
        Ok(LuaValue::String(s)) => s.to_str().unwrap().to_string(),
        Ok(LuaValue::Nil) => String::new(),
        Ok(LuaValue::Thread(t)) => match t.into_async::<_, LuaValue>(()).await {
            Err(e) => format_error(&e),
            Ok(LuaValue::String(s)) => s.to_str().unwrap().to_string(),
            Ok(LuaValue::Nil) => String::new(),
            Ok(v) => format!("{:?}", v),
        },
        Ok(v) => format!("{:?}", v),
    };

    if !msg.is_empty() {
        if let Err(e) = notify_tx.send(msg).await {
            println!("spawn_eval_task -> notify_tx.send: {}", e);
        }
    }
}

async fn sleep(_lua: &Lua, n: u64) -> LuaResult<&'static str> {
    tokio::time::sleep(Duration::from_millis(n)).await;
    Ok("done")
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

    lua.globals()
        .set("sleep", lua.create_async_function(sleep).unwrap())
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
    lua: Lua,
    uuid: Uuid,
    notify_tx: mpsc::Sender<String>,
    line_rx: async_channel::Receiver<String>,
    db: Arc<RwLock<Database>>,
) {
    tokio::task::spawn_local(async move {
        let conndata = ConnData {
            player_object: uuid,
        };
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
                        Ok((chunk_name, lua_code)) => {
                            eval(&lua, chunk_name, lua_code, &notify_tx).await
                        }
                        Err(s) => {
                            notify_tx.send(s).await.ok();
                        }
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
            .and_then(|l| lock.get(&l).ok())
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
        .collect::<Vec<_>>();

    // Not very efficient but /shrug
    let mut this_args = vec!["this".to_string()];
    this_args.extend(args.clone());

    Ok((
        format!("{}:{}({})", this.name(), verb_name, args.join(", ")),
        format!(
            "
            player = toobj({:?}):unwrap()
            dobjstr = {:?}
            argstr = {:?}
            local this = toobj({:?}):unwrap()
            this[{:?}]({})
        ",
            player_uuid.to_string(),
            dobjstr,
            argstr,
            this_uuid,
            verb_name,
            this_args.join(", ")
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

fn move_notify_channel(
    our_txs: Arc<RwLock<HashMap<Uuid, mpsc::Sender<String>>>>,
    old: &Uuid,
    new: Uuid,
) {
    let mut lock = our_txs.write().unwrap();
    let val = lock.remove(old).unwrap();
    lock.insert(new, val);
}

async fn do_login_command(
    lua: &Lua,
    line_rx: async_channel::Receiver<String>,
    notify_tx: mpsc::Sender<String>,
    db: Arc<RwLock<Database>>,
) -> Option<Uuid> {
    notify_tx
        .send("Ohai! Type 'connect <username>' to get started.".to_string())
        .await
        .unwrap();
    loop {
        let line = match line_rx.recv().await {
            Ok(l) => l,
            Err(e) => {
                // TODO err logging
                println!("{}", e);
                return None;
            }
        };
        println!("< {}", line);

        let uuid = &CONNDATA.get().player_object;
        let command_opt = {
            let lock = db.read().unwrap();
            Command::parse(&line, uuid, &lock)
        };
        let command = match command_opt {
            Some(cmd) => cmd,
            None => {
                println!("do_login_command: failed to parse: {}", line);
                continue;
            }
        };
        let mut args = vec![command.verb().to_string()];
        args.extend(command.args().clone());
        let argstr = args
            .iter()
            .map(|s| format!("{:?}", s))
            .collect::<Vec<_>>()
            .join(", ");
        let short_code = format!("S:do_login_command({})", argstr);
        let full_code = format!(
            "
        player = db[{:?}]
        return {}
        ",
            uuid.to_string(),
            short_code
        );

        println!("{}", full_code);
        let retval = lua
            .load(&full_code)
            .set_name(&short_code)
            .unwrap()
            .eval::<LuaValue>();
        let msg_opt = match retval {
            Ok(LuaValue::String(s)) => {
                return Some(DatabaseProxy::parse_uuid_old(s.to_str().unwrap()).unwrap())
            }
            Ok(LuaValue::Nil) => None,
            Ok(v) => Some(format!("{:?}", v)),
            Err(e) => Some(format!("{}", e)),
        };
        if let Some(msg) = msg_opt {
            if let Err(e) = notify_tx.send(msg).await {
                println!("do_login_command notify failed: {}", e);
            }
        }
    }
}

fn inject_notify_function(lua: &Lua, notify_txs: Arc<RwLock<HashMap<Uuid, mpsc::Sender<String>>>>) {
    lua.globals()
        .set(
            "_server_notify",
            lua.create_function(move |_lua, (uuid, msg): (String, String)| {
                let lock = notify_txs.read().unwrap();
                if let Some(tx) = lock.get(&DatabaseProxy::parse_uuid_old(&uuid)?) {
                    // TODO handle buffer full
                    return tx.try_send(msg).map_err(LuaError::external);
                }
                Ok(())
            })
            .unwrap(),
        )
        .unwrap();
}
