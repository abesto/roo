use anyhow::Result;
use async_channel::{Receiver, Sender};
use database::{Database, SharedDatabase};
use rhai::{Engine, Scope};
use structopt::StructOpt;
use tokio::{
    self,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
};

#[macro_use]
mod error;
mod api;
mod database;

#[derive(Debug, StructOpt)]
struct Opt {
    input_db_file: String,
    output_db_file: String,

    #[structopt(default_value = "8888")]
    port: u16,
}

struct Context {
    exit_tx: ExitSender,
}

impl Context {
    #[must_use]
    fn new(exit_tx: ExitSender) -> Self {
        Self { exit_tx }
    }
}

type ExitSender = tokio::sync::mpsc::UnboundedSender<()>;
type SharedContext = std::sync::Arc<parking_lot::RwLock<Context>>;

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    let listener = listen(opt.port).await?;
    let database = Database::load(&opt.input_db_file)
        .map(|db| {
            eprintln!("Loaded database from {}", opt.input_db_file);
            db
        })
        .unwrap_or_else(|e| {
            eprintln!("Failed to load database from {}: {}", opt.input_db_file, e);
            eprintln!("Creating new database...");
            Database::new()
        })
        .share();

    let _dump_database_on_drop = DumpDatabaseOnDrop::new(database.clone(), &opt.output_db_file);

    // TODO: push this Context down somehow so that we can exit from a command
    let (exit_tx, mut exit_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let context: SharedContext =
        std::sync::Arc::new(parking_lot::RwLock::new(Context::new(exit_tx)));
    {
        let handler_context = context.clone();
        ctrlc::set_handler(move || {
            eprintln!("Caught Ctrl-C");
            handler_context
                .write()
                .exit_tx
                .send(())
                .expect("Failed to send exit signal");
        })
        .expect("Error setting Ctrl-C handler");
    }

    loop {
        tokio::select! {
            _ = exit_rx.recv() => {
                eprintln!("Exiting...");
                break;
            },
            Ok((socket, _)) = listener.accept() => {
                handle_connection(socket, database.clone());
            }
        }
    }

    Ok(())
}

struct DumpDatabaseOnDrop {
    database: SharedDatabase,
    output_db_file: String,
}

impl DumpDatabaseOnDrop {
    fn new(database: SharedDatabase, output_db_file: &str) -> Self {
        Self {
            database,
            output_db_file: output_db_file.to_string(),
        }
    }
}

impl Drop for DumpDatabaseOnDrop {
    fn drop(&mut self) {
        eprintln!("Performing final DB dump to {}", self.output_db_file);
        let _ = self
            .database
            .read()
            .save(&self.output_db_file)
            .map_err(|e| {
                eprintln!("Failed to save database to {}: {}", self.output_db_file, e);
            });
    }
}

fn handle_connection(socket: TcpStream, database: SharedDatabase) {
    tokio::spawn(async move {
        let (read, write) = socket.into_split();

        let mut engine = Engine::new();
        engine.set_max_expr_depths(64, 64);
        api::register_api(&mut engine, database);

        let (line_tx, line_rx) = async_channel::unbounded::<String>();
        spawn_read_task(read, line_tx);
        spawn_processing_task(engine, write, line_rx);
    });
}

async fn listen(port: u16) -> Result<TcpListener> {
    println!("Server started");
    let addr = format!("127.0.0.1:{}", port);
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
                let code = format!("toliteral(eval({:?}))", stripped);
                let result = engine.eval_with_scope::<String>(&mut scope, &code);
                let maybe_msg = match result {
                    Ok(x) if !x.is_empty() => Some(format!("=> {}\r\n", x)),
                    Ok(_) => None,
                    Err(e) => Some(format!("{}\r\n", e)),
                };

                if let Some(msg) = maybe_msg {
                    write.write_all(msg.as_bytes()).await.unwrap();
                }
            }
        }
    });
}
