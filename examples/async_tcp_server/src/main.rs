use futures::StreamExt;
use std::io;
use wasmedge_async::{spawn, AsyncWriteExt, Executor, TcpListener, AsyncReadExt};

async fn listener_test() -> io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("1235".to_string());
    println!("Listen 127.0.0.1:{}", port);
    let mut listener = TcpListener::bind(format!("127.0.0.1:{}", port), true)?;
    while let Some(ret) = listener.next().await {
        if let Ok((mut stream, addr)) = ret {
            println!("Accept a new connection from {} successfully", addr);
            let f = async move {
                let mut response = [0 as u8;11];
                let _ = stream.read(&mut response).await.unwrap();
                println!("Get response: {}", std::str::from_utf8(&response).unwrap());
                stream.write(b"hello").await.unwrap();
            };
            spawn(f);
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let mut executor = Executor::new();
    executor.block_on(listener_test)??;
    Ok(())
}
