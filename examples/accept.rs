//! A loop accept example.
//!
//! Run the example and `nc 127.0.0.1 50002` in another shell.

use std::time::Duration;

use snowfallio::net::TcpListener;

#[snowfallio::main(enable_timer = true)]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:50002").unwrap();
    snowfallio::spawn(async {
        loop {
            snowfallio::time::sleep(Duration::from_secs(1)).await;
            println!("tik tok");
        }
    });
    println!("listening");
    loop {
        println!("waiting...");
        let incoming = listener.accept().await;
        match incoming {
            Ok((_, addr)) => {
                println!("accepted a connection from {addr}");
            }
            Err(e) => {
                println!("accepted connection failed: {e}");
                return;
            }
        }
    }
}
