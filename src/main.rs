use tokio::io::AsyncWriteExt;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use std::error::Error;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Server started");
    let addr = "127.0.0.1:8888";
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        let (socket, _) = listener.accept().await?;
        let (read, mut write) = socket.into_split();
        let mut lines = BufReader::new(read).lines();
        tokio::spawn(async move {
            while let Some(line) = lines.next_line().await.unwrap() {
                let response = line + "\n";
                write.write_all(response.as_bytes()).await.unwrap();
            }
        });
    }
}
