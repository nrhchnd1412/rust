use std::sync::{Arc, mpsc::channel};
use parking_lot::Mutex;
use std::thread;

const N:usize = 10;

fn main() {
    let data = Arc::new(Mutex::new(0));
    let (tx, rx) = channel();

    for i in 0..N {
        let (data,tx)=(Arc::clone(&data), tx.clone());
        thread::spawn(move || {
            let mut data = data.lock();
            println!("thread no {}",i);
            *data += 1;
            if *data == N {
                tx.send("done 10").unwrap();
            }
        });
    }
    let resp = rx.recv().unwrap();
    println!("{}", resp);
}
