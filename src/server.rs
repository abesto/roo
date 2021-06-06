use mlua::prelude::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::command::{parse_command, Command};
use crate::database::{Database, DatabaseProxy, Object, Verb, World};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

#[derive(Copy, Clone)]
pub struct ConnData {
    player_object: Uuid,
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

#[tokio::main]
pub async fn run_server(world: World) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8888").await?;
    let txs: Arc<RwLock<HashMap<Uuid, Sender<String>>>> = Arc::new(RwLock::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await?;
        let our_txs = txs.clone();
        let lua = world.lua();
        let db = world.db();

        tokio::spawn(async move {
            let (read, mut write) = socket.into_split();

            let uuid_str = lua
                .load("system:do_login_command()")
                .set_name("system:do_login_command()")
                .unwrap()
                .eval::<String>()
                .unwrap();
            let uuid = DatabaseProxy::parse_uuid(&uuid_str).unwrap();

            {
                let notify_txs = our_txs.clone();
                lua.globals()
                    .set(
                        "notify",
                        lua.create_function(move |_lua, (uuid, msg): (String, String)| {
                            let lock = notify_txs.read().unwrap();
                            if let Some(tx) = lock.get(&DatabaseProxy::parse_uuid(&uuid)?) {
                                // TODO handle buffer full
                                return tx
                                    .try_send(msg)
                                    .map_err(|e| LuaError::RuntimeError(e.to_string()));
                            }
                            Ok(())
                        })
                        .unwrap(),
                    )
                    .unwrap();
            }

            let (tx, mut rx) = mpsc::channel::<String>(100);
            {
                let mut lock = our_txs.write().unwrap();
                lock.insert(uuid, tx.clone());
            }

            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    write
                        .write_all(format!("{}\r\n", msg).as_bytes())
                        .await
                        .unwrap();
                }
            });

            tokio::spawn(async move {
                let conndata = ConnData {
                    player_object: uuid,
                };

                CONNDATA
                    .scope(conndata, async move {
                        let mut lines = BufReader::new(read).lines();
                        tx.send("Hai!".to_string()).await.unwrap();
                        while let Some(line) = lines.next_line().await? {
                            let maybe_msg = if let Some(stripped) = line.strip_prefix(';') {
                                // If it starts with a ";", run it as Lua
                                match lua.load(stripped).eval::<LuaValue>() {
                                    Err(e) => Some(e.to_string()),
                                    Ok(LuaValue::String(s)) => {
                                        Some(s.to_str().unwrap().to_string())
                                    }
                                    Ok(v) => Some(format!("{:?}", v)),
                                }
                            } else {
                                // Otherwise, as a ROO command
                                {
                                    let lock = db.read().unwrap();
                                    parse_command(&line)
                                        .ok_or("Failed to parse command".to_string())
                                        .and_then(|command| -> Result<(), String> {
                                            // Find who does what
                                            let player: &Object =
                                                lock.get(&CONNDATA.get().player_object)?;
                                            let location: Option<&Object> =
                                                player.location().and_then(|l| lock.get(l).ok());
                                            let (this, verb) = first_matching_verb(
                                                &lock,
                                                &command,
                                                vec![Some(player), location],
                                            )?
                                            .ok_or_else(|| "Unknown verb".to_string())?;

                                            // Set up arguments
                                            let lua_player = get_object_proxy(&lua, player.uuid());
                                            lua.globals().set("player", lua_player).unwrap();
                                            let args = command.to_args().to_lua(&lua);

                                            // Execute verb
                                            let cmd =
                                                &format!("db[\"{}\"].{}", this.uuid(), verb.name());
                                            lua.load(cmd)
                                                .set_name(cmd)
                                                .unwrap()
                                                .eval::<LuaFunction>()
                                                .and_then(|f| f.call(args))
                                                .map_err(|e| e.to_string())
                                        })
                                }
                                .map_or_else(|e| Some(e.to_string()), |_| None)
                            };
                            if let Some(msg) = maybe_msg {
                                tx.send(msg.to_string()).await.unwrap();
                            }
                        }
                        Ok::<(), std::io::Error>(())
                    })
                    .await
                    .unwrap();
            });
        });
    }

    // TODO notify players when a player in the same room disconnects
}
