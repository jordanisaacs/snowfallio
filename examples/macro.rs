//! Use macro to run async main

#[snowfallio::main(entries = 512)]
async fn main() {
    println!("will sleep about 1 sec");

    let begin = std::time::Instant::now();
    snowfallio::time::sleep(snowfallio::time::Duration::from_secs(1)).await;
    let eps = std::time::Instant::now().saturating_duration_since(begin);

    println!("elapsed: {}ms", eps.as_millis());
}
