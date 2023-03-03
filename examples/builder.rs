//! Except for using macro, You have 3 ways to start the runtime manually.

fn main() {
    // 1. Create runtime and block_on normally
    let mut rt = snowfallio::RuntimeBuilder::<snowfallio::IoUringDriver>::new()
        .build()
        .unwrap();
    rt.block_on(async {
        println!("it works1!");
    });

    // 2. Create runtime with custom options and block_on
    let mut rt = snowfallio::RuntimeBuilder::<snowfallio::IoUringDriver>::new()
        .with_entries(256)
        .enable_timer()
        .build()
        .unwrap();
    rt.block_on(async {
        println!("it works2!");
    });

    // 3. Use `start` directly: equivalent to default runtime builder and block_on
    snowfallio::start::<snowfallio::IoUringDriver, _>(async {
        println!("it works3!");
    });
}
