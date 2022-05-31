use futures::StreamExt;
use std::io;
use tokio::io::{AsyncWriteExt};
use wasmedge_async::{spawn, Executor, TcpListener};

async fn listener_test() -> io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("1235".to_string());
    println!("Listen 127.0.0.1:{}", port);
    let mut listener = TcpListener::bind(format!("127.0.0.1:{}", port), true)?;
    while let Some(ret) = listener.next().await {
        if let Ok((mut stream, addr)) = ret {
            println!("Accept a new connection from {} successfully", addr);
            let f = async move {
                stream.write_all(b"hello").await.unwrap();
            };
            spawn(f);
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let mut executor = Executor::new();
    async fn listen() -> io::Result<()> {
        println!("Before listening ...");
        spawn(async {
            println!("Dummy task!");
        });
        listener_test().await?;
        Ok(())
    }
    executor.block_on(listen)?;
    Ok(())
}
