use mlua::prelude::LuaValue;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::command::parse_command;
use crate::database::{DatabaseProxy, World};

#[derive(Copy, Clone)]
pub struct ConnData {
    player_object: Uuid,
}

tokio::task_local! {
    pub static CONNDATA: ConnData
}

#[tokio::main]
pub async fn run_server(world: World) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8888").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        let lua = world.lua();
        let db = world.db();

        let uuid_str = lua
            .load(include_str!("lua/spawn_player.lua"))
            .set_name("spawn_player")
            .unwrap()
            .eval::<String>()
            .unwrap();

        tokio::spawn(async move {
            let conndata = ConnData {
                player_object: DatabaseProxy::parse_uuid(&uuid_str).unwrap(),
            };

            CONNDATA
                .scope(conndata, async move {
                    let (read, mut write) = socket.split();
                    let mut lines = BufReader::new(read).lines();
                    write.write_all(b"Hai!\r\n$ ").await?;
                    while let Some(line) = lines.next_line().await? {
                        let msg = if let Some(stripped) = line.strip_prefix('>') {
                            // If it starts with a ">", run it as Lua
                            match lua.load(stripped).eval::<LuaValue>() {
                                Err(e) => e.to_string(),
                                Ok(LuaValue::String(s)) => s.to_str().unwrap().to_string(),
                                Ok(v) => format!("{:?}", v),
                            }
                        } else {
                            // Otherwise, as a ROO command
                            {
                                let lock = db.read().unwrap();
                                parse_command(&line).and_then(|command| {
                                    let verb = command.verb();
                                    let player = lock.get(&CONNDATA.get().player_object)?;
                                    let location = lock.get(player.location()?)?;
                                    match verb {
                                        "look" => location
                                            .properties
                                            .get("description")
                                            .cloned()
                                            .or_else(|| Some(String::new())),
                                        _ => None,
                                    }
                                })
                            }
                            .unwrap_or_else(|| "I didn't understand that.".to_string())
                        };
                        write.write_all((msg + "\r\n$ ").as_bytes()).await?;
                    }
                    Ok::<(), std::io::Error>(())
                })
                .await
        });
    }
}
