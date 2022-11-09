#![allow(unused_imports)]
use std::collections::HashSet;
use std::io::{stdin, stdout, Write};
use std::{thread, time};
use std::sync::{Arc, Mutex};
use triple_buffer::triple_buffer;
use obmerge::orderbook::{OrderLine, OrderBook};
use obmerge::exchanges::{obbinance::OBBinance, obbitstamp::OBBitstamp};
use obmerge::proto::order_book::Summary;
use obmerge::server;
use std::mem::take;
use tonic::{transport::Server, Request, Response, Status, Streaming};
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use std::process;

type BookType = Arc<RwLock<Vec<triple_buffer::Output<OrderBook>>>>;

//allowed symbols
const ALLSYM: [&'static str; 3] = ["ethbtc", "ethusdt", "btcusdt"];


async fn clear(handles: &mut Vec<JoinHandle<()>>, books_lock: &mut BookType) {
    books_lock.write().await.clear();
    for handle in take(handles) {
        handle.abort();
    }
    handles.clear();
}

fn book_zero() -> OrderBook{
    OrderBook{
        spread: 0.0,
        asks: Vec::new(),
        bids: Vec::new(),
    }
}


// This function will repeatedly await a signal from any exchange thread, and then will publish a combined orderbook to the Server
async fn output_loop(mut exch_rx: mpsc::Receiver<usize>, server_tx: broadcast::Sender<Option<Summary>>, books_lock: BookType) {
    loop{
        if let Some(id) = exch_rx.recv().await {
            if id == 0 {
                // copy all exchage data locally and ensure that we can build and publish an orderbook
                let mut books = books_lock.write().await;
                let n = books.len();
                if n == 0{
                    continue;
                }
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

                // get the 10 best bids 
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

                // get the 10 best asks
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

                // get the spread
                res.spread = res.asks[0].price - res.bids[0].price;

                // send the resulting orderbook to the Server
                let _ = server_tx.send(Some(res.to_summary()));
            }
        }
    }

}

#[tokio::main]
async fn main() {
    // create the server broadcast channel and start the server in a new thread
    let (server_tx, _server_rx) = broadcast::channel::<Option<Summary>>(10);
    let server_tx_clone = server_tx.clone();
    tokio::spawn(async move {
        let _ = server::start(server_tx_clone).await;
    });

    //create the data structure containing the exchange orderbooks
    let mut books_lock: BookType = Arc::new(RwLock::new(Vec::new()));
    let books_lock_clone = books_lock.clone();

    // create the exchange mpsc channel and start listening for orderbook update events in a new thread
    let (tx, rx) = mpsc::channel::<usize>(1);
    tokio::spawn(async move {
        output_loop(rx, server_tx, books_lock_clone).await;
    });

    // initialize values for the CLI and make a set of allowed symbols
    let mut buffer: String = String::new();
    let status: Vec<String> = vec![String::from("offline"), String::from("online")];
    let mut is_online = 0;
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    let mut hashsym = HashSet::new();
    for x in ALLSYM {
        hashsym.insert(x);
    }

    // start the CLI
    loop{
        buffer.clear();

        print!("{}>", status[is_online]);
        stdout().flush().expect("");
        stdin().read_line(&mut buffer).expect("");
        let mut iter = buffer.split_whitespace();
        let word = iter.next();

        match word.unwrap_or("continue"){
            "continue" => {}
            "quit" => {
                process::exit(0);
            },
            "start" => {
                if is_online == 1 {
                    println!("Can't start while online");
                    continue;
                }
                //get trading symbol
                let pair = match iter.next(){
                    None => {
                        println!("Please provide a symbol");
                        continue;
                    }
                    Some(x) => {
                        if !hashsym.contains(x) {
                            println!("Symbol not allowed");
                            continue;
                        }
                        x
                    }
                };

                // Connect to Binance
                let msg = "Connecting to Binance...";
                let (cbin, cbout) = triple_buffer(&book_zero());
                books_lock.write().await.push(cbout);
                let ctx = tx.clone();
                let cpair = String::from(pair);
                handles.push(tokio::spawn(async move {
                    let exch = OBBinance::new(cbin, cpair);
                    match exch {
                        Ok(mut x) => {
                            println!("{}Success!", msg);
                            x.listen(ctx).await;
                        }
                        Err(x) => {
                            println!("{}Failed with error: {}", msg, x);
                        }
                    }
                }));

                // Connect to Bitstamp
                let msg = "Connecting to Bitstamp...";
                let (cbin, cbout) = triple_buffer(&book_zero());
                books_lock.write().await.push(cbout);
                let ctx = tx.clone();
                let cpair = String::from(pair);
                handles.push(tokio::spawn(async move {
                    let exch = OBBitstamp::new(cbin, cpair);
                    match exch {
                        Ok(mut x) => {
                            println!("{}Success!", msg);
                            x.listen(ctx).await;
                        }
                        Err(x) => {
                            println!("{}Failed with error: {}", msg, x);
                        }
                    }
                }));
                
                is_online = 1;  // exchange connections are now online

                thread::sleep(time::Duration::new(2, 0));  // don't mess up CLI output

            },
            "stop" => {
                clear(&mut handles, &mut books_lock).await;
                is_online = 0;
            }
            _ => {
                println!("Unimplemented Command");
            },
        }
    };
    
}