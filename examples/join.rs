//! Since futures only executed when it is polled or awaited,
//! this example shows how to await multiple futures at the same time.
//! (Another way is spawning them and await the JoinHandle.)

#[snowfallio::main]
async fn main() {
    println!("directly await ready_now: {}", ready_now().await);

    let to_spawn = snowfallio::spawn(ready_now());
    println!("spawn await ready_now: {:?}", to_spawn.await);

    snowfallio::join!(ready_now(), ready_now());
    println!("snowfallio::join two tasks");
}

async fn ready_now() -> u8 {
    7
}
