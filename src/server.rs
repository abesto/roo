use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use mlua::prelude::LuaValue;

use crate::database;

#[tokio::main]
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let world = database::World::new();

    let listener = TcpListener::bind("127.0.0.1:8888").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        let lua = world.lua();

        tokio::spawn(async move {
            let (read, mut write) = socket.split();
            let mut lines = BufReader::new(read).lines();

            write.write_all(b"Hai!\r\n$ ").await?;

            // In a loop, read data from the socket and write the data back.
            loop {
                while let Some(line) = lines.next_line().await? {
                    let msg = match lua.load(&line).eval::<LuaValue>() {
                        Err(e) => e.to_string(),
                        Ok(LuaValue::String(s)) => s.to_str().unwrap().to_string(),
                        Ok(v) => format!("{:?}", v),
                    };
                    write.write_all((msg + "\r\n$ ").as_bytes()).await?;
                }
                return Ok::<(), std::io::Error>(());
            }
        });
    }
}
