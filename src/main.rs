#![allow(unused_imports)]
use std::io::{stdin, stdout, Write};
use std::{thread, time};
use std::sync::{Arc, Mutex};
use triple_buffer::triple_buffer;
use obmerge::orderbook::{OrderLine, OrderBook};
use obmerge::exchanges::{obbinance::OBBinance, obbitstamp::OBBitstamp};
use obmerge::proto::order_book::Summary;
use obmerge::server;
use std::mem::take;
use crossbeam_channel::{unbounded, TryRecvError, Receiver, Sender};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use tokio::sync::broadcast;

fn clear(exchid: &mut Vec<usize>, tx: &Sender<usize>, handles: &mut Vec<thread::JoinHandle<()>>, books: &mut Vec<triple_buffer::Output<OrderBook>>) {
    for id in exchid.iter() {
        tx.send(*id).expect("");
    }
    exchid.clear();
    for handle in take(handles) {
        handle.join().unwrap();
    }
    handles.clear();
    books.clear();
}

fn book_zero() -> OrderBook{
    OrderBook{
        spread: 0.0,
        asks: Vec::new(),
        bids: Vec::new(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let (obtx, _) = broadcast::channel::<Summary>(9);
    let cobtx = obtx.clone();
    tokio::spawn(async move {
        server::start(cobtx).await.unwrap();
    });

    let mut buffer: String = String::new();
    let status: Vec<String> = vec![String::from("offline"), String::from("online")];
    let mut ps = 0;

    let mut books = Vec::new();

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    let (mut tx, mut rx) = unbounded();
    let mut exchid: Vec<usize> = Vec::new();
    loop{
        buffer.clear();
        print!("{}>", status[ps]);
        stdout().flush().expect("");
        stdin().read_line(&mut buffer).expect("");
        let mut iter = buffer.split_whitespace();
        let word = iter.next();
        if word == None{
            continue;
        }
        match word.unwrap(){
            "quit" => {
                clear(&mut exchid, &tx, &mut handles, &mut books);
                break;
            },
            "start" => {
                ps = 1;

                //get trading symbol
                let pair = iter.next();
                if pair == None {
                    println!("Please provide a symbol");
                    continue;
                }

                //Connect to Binance
                print!("Connecting to Binance...");
                let (cbin, cbout) = triple_buffer(&book_zero());
                books.push(cbout);
                let crx = rx.clone();
                let id = 0;
                exchid.push(id);
                let cpair = String::from(pair.unwrap());
                handles.push(thread::spawn(move || {
                    let mut obb: OBBinance = OBBinance::new(cbin, cpair);
                    obb.listen(crx, id);
                }));
                let aux: Result<(), String> = Ok(());
                match aux {
                    Ok(()) => {
                        println!("Success!");
                    }
                    Err(x) => {
                        println!("Failed with error {}", x);
                        continue;
                    }
                }

                //Connect to Bitstamp
                print!("Connecting to Bitstamp...");
                let (cbin, cbout) = triple_buffer(&book_zero());
                books.push(cbout);
                let crx = rx.clone();
                let id = 1;
                exchid.push(id);
                let cpair = String::from(pair.unwrap());
                handles.push(thread::spawn(move || {
                    let mut obb: OBBitstamp = OBBitstamp::new(cbin, cpair);
                    obb.listen(crx, id);
                }));
                let aux: Result<(), String> = Ok(());
                match aux {
                    Ok(()) => {
                        println!("Success!");
                    }
                    Err(x) => {
                        println!("Failed with error {}", x);
                        continue;
                    }
                }

                thread::sleep(time::Duration::new(2, 0));

            },
            "stop" => {
                clear(&mut exchid, &tx, &mut handles, &mut books);
                (tx, rx) = unbounded();
                ps = 0;
            }
            "check" => {
                let n = books.len();
                let mut ob: Vec<OrderBook> = Vec::new();
                let mut tot: usize = 0;
                let mut res = book_zero();
                for i in 0..n{
                    ob.push(books[i].read().clone());
                    tot += ob.last().unwrap().bids.len();
                }
                if tot < 10{
                    println!("Not enough bids/asks");
                    continue;
                }
                let mut p = vec![0; n];
                for _ in 0..10 {
                    let mut best = OrderLine { exchange: String::new(), price: 0.0, amount: 0.0 };
                    let mut bp: usize = 0;
                    for i in 0..n {
                        if p[i] >= ob[i].bids.len(){
                            continue;
                        }
                        if best.smaller(&ob[i].bids[p[i]]){
                            best = ob[i].bids[p[i]].clone();
                            bp = i; 
                        }
                    }
                    res.bids.push(ob[bp].bids[p[bp]].clone());
                    p[bp] += 1;
                }

                let mut p = vec![0; n];
                for _ in 0..10 {
                    let mut best = OrderLine { exchange: String::new(), price: 0.0, amount: 0.0 };
                    let mut bp: usize = 0;
                    for i in 0..n {
                        if p[i] >= ob[i].asks.len(){
                            continue;
                        }
                        if best.greater(&ob[i].asks[p[i]]){
                            best = ob[i].asks[p[i]].clone();
                            bp = i; 
                        }
                    }
                    res.asks.push(ob[bp].asks[p[bp]].clone());
                    p[bp] += 1;
                }

                res.spread = res.asks[0].price - res.bids[0].price;
                println!("{}", serde_json::to_string(&res).unwrap());
                obtx.send(res.to_summary()).unwrap();
            }
            _ => {
                println!("Unknown Command");
            },
        }
    }
    Ok(())
}