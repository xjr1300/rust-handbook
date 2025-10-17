use std::thread;
use std::time::Duration;

fn main() {
    // メインスレッドのハンドルを取得
    let main_handle = thread::current();

    let child_handle = thread::spawn(move || {
        // 3秒後にメインスレッドを再開
        println!("child thread: sleeping in three seconds");
        thread::sleep(Duration::from_secs(3));
        println!("child thread: git a token to main thread (unpark)");
        main_handle.unpark();
    });

    println!("main thread: blocking for park");
    thread::park(); // トークンがないためブロックされる
    println!("main thread: take a token in child thread, so main thread is resumed");

    child_handle.join().unwrap();
}
