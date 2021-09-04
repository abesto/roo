use anyhow::Result;
use async_channel::{Receiver, Sender};
use database::{Database, SharedDatabase};
use rhai::{Dynamic, Engine, EvalAltResult, Scope};
use tokio::{
    self,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
};

use crate::{api::ObjectProxy, error::Error};

#[macro_use]
mod error;
mod api;
mod database;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = listen().await?;
    let database = Database::new().share();

    loop {
        let (socket, _) = listener.accept().await?;
        handle_connection(socket, database.clone());
    }
}

fn handle_connection(socket: TcpStream, database: SharedDatabase) {
    tokio::spawn(async move {
        let (read, write) = socket.into_split();

        let mut engine = Engine::new();
        api::register_api(&mut engine, database);

        let (line_tx, line_rx) = async_channel::unbounded::<String>();
        spawn_read_task(read, line_tx);
        spawn_processing_task(engine, write, line_rx);
    });
}

async fn listen() -> Result<TcpListener> {
    println!("Server started");
    let addr = "127.0.0.1:8888";
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);
    Ok(listener)
}

fn spawn_read_task(read: OwnedReadHalf, line_tx: Sender<String>) {
    let mut lines = BufReader::new(read).lines();
    tokio::spawn(async move {
        while let Some(line) = lines.next_line().await.unwrap() {
            line_tx.send(line).await.unwrap();
        }
    });
}

fn spawn_processing_task(engine: Engine, mut write: OwnedWriteHalf, line_rx: Receiver<String>) {
    tokio::spawn(async move {
        let mut scope = Scope::new();
        loop {
            let line = match line_rx.recv().await {
                Ok(l) => l,
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            };

            println!("< {}", line);
            if let Some(stripped) = line.strip_prefix(';') {
                // TODO this will need to move into the core, and we'll just translate to eval() here
                let result = engine.eval_with_scope::<Dynamic>(&mut scope, &stripped);
                let maybe_msg = match result {
                    Ok(x) => {
                        if !x.is::<()>() {
                            Some(format!("=> {}\r\n", x))
                        } else {
                            None
                        }
                    }
                    Err(e) => match e.as_ref() {
                        EvalAltResult::ErrorRuntime(d, _) => {
                            if d.is::<Error>() {
                                Some(format!("=> {}\r\n", d.clone().cast::<Error>()))
                            } else {
                                Some(format!("{}\r\n", d))
                            }
                        }
                        _ => Some(format!("{}\r\n", e)),
                    },
                };

                if let Some(msg) = maybe_msg {
                    write.write_all(msg.as_bytes()).await.unwrap();
                }
            }
        }
    });
}
