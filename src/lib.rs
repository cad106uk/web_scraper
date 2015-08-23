use std::thread;

fn worker() -> i64 {
    let mut x: i64 = 0;
    while x < 15_000_000 {
        x += 1
    }
    x
}

#[no_mangle]
pub extern fn process() {
    let handles: Vec<_> = (0..10).map(
        |_| {thread::spawn(|| worker())
    }).collect();

    for h in handles {
        let res = h.join().map_err(|_| "Could not join a thread!");
        println!("Thread finished with count={}", res.unwrap());
    }
    println!("Done");
}


#[test]
fn it_works() {
    assert_eq!(worker(), 15_000_000);
}
