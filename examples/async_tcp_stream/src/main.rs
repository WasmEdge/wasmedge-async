use std::io;
use wasmedge_async::{spawn, Executor, TcpStream, AsyncReadExt, AsyncWriteExt};
use futures::future::join_all;

async fn stream_test() -> io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("1235".to_string());
    println!("Connect to 127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
    // send the message, remember to add '\n'
    stream.write(b"hello world\n").await?;
    stream.flush().await?;
    println!("Flush.");
    let mut response = [0 as u8;5];
    let length = stream.read(&mut response).await?;
    assert_eq!(length, 5);
    assert_eq!(&response, "hello".as_bytes());
    println!("Get response");
    Ok(())
}

fn main() -> io::Result<()> {
    let mut executor = Executor::new();
    async fn connect() -> io::Result<()> {
        println!("Before connecting ...");
        spawn(async {
            println!("Dummy task!");
        });
        join_all(vec![stream_test(), stream_test(), stream_test()]).await;
        println!("Finish request!");
        Ok(())
    }
    executor.block_on(connect)?;
    Ok(())
}
