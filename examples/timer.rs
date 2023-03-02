//! You may not call thread::sleep in async runtime, which will block the whole
//! thread. Instead, you should use snowfallio::time provided functions.

use std::time::Duration;

#[snowfallio::main(enable_timer = true)]
async fn main() {
    loop {
        snowfallio::time::sleep(Duration::from_secs(1)).await;
        println!("balabala");
        snowfallio::time::sleep(Duration::from_secs(1)).await;
        println!("abaaba");
    }
}
