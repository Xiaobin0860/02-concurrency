//
// std mpsc|thread, rand exit producer

use std::{sync::mpsc, thread};

use anyhow::anyhow;

const PRODUCER_COUNT: usize = 4;

#[derive(Debug)]
struct Msg {
    idx: usize,
    val: usize,
}

fn main() -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel();
    //
    for i in 0..PRODUCER_COUNT {
        let tx = tx.clone();
        thread::spawn(move || produser(i, tx));
    }
    drop(tx); // close tx, so rx will return

    let handle = thread::spawn(move || consumer(rx));
    let val = handle.join().map_err(|e| anyhow!("join error: {e:?}"))?;
    println!("val={val}");

    Ok(())
}

fn produser(idx: usize, tx: mpsc::Sender<Msg>) -> anyhow::Result<()> {
    loop {
        let val = rand::random::<usize>() % 100;
        println!("producer {idx} val={val}");
        if val % 5 == 0 {
            break;
        }
        let msg = Msg { idx, val };
        println!("producer {idx} send msg={msg:?} ...");
        if let Err(e) = tx.send(msg) {
            eprintln!("send error: {e:?}");
            break;
        }
    }
    println!("producer {idx} exit");
    Ok(())
}

fn consumer(rx: mpsc::Receiver<Msg>) -> usize {
    let mut value = 0;
    for Msg { idx, val } in rx {
        println!("recv from {idx}, val={val}");
        value += val;
    }
    value
}
