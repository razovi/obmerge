use std::net::TcpStream;
use crate::orderbook::{OrderLine, OrderBook};
use tungstenite::{connect, WebSocket, stream::MaybeTlsStream};
use url::Url;
use tokio::sync::mpsc::Sender;
use serde_json;
use serde::Deserialize;


pub struct OBBinance{
    book: triple_buffer::Input<OrderBook>,
    pub status: Result<(), String>,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct BinanceBook{
    lastUpdateId: u64,
    bids: Vec<[String; 2]>,
    asks: Vec<[String; 2]>,
}

impl OBBinance {
    pub fn new(book: triple_buffer::Input<OrderBook>, pair: String) -> Self{
        let url = format!("wss://stream.binance.com:9443/ws/{}@depth20@100ms/ws", pair);
        let r = connect(Url::parse(&url).unwrap());
        let mut socket = None;
        let ssocket;
        let response;
        let status = match r {
            Ok(x) => {
                (ssocket, response) = x;
                socket = Some(ssocket);
                println!("Connected to binance stream.");
                println!("HTTP status code: {}", response.status());
                Ok(())
            }
            Err(_) => {
                Err(String::from("Can't connect"))
            }
        };
        OBBinance{
            book,
            status,
            socket,
        }
    }
    pub async fn listen(&mut self, tx: Sender<usize>) {
        let mut fallback = BinanceBook{lastUpdateId: 0, bids: Vec::new(), asks: Vec::new()};
        loop{
            let msg = self.socket.as_mut().unwrap().read_message().expect("Error reading message");
            //println!("Received: {}", msg);

            let mut o: OrderBook = OrderBook{
                spread: 0.0,
                asks: Vec::new(),
                bids: Vec::new(),
            };

            let b: BinanceBook = serde_json::from_str(&msg.into_text().unwrap()[..]).unwrap_or(fallback);
            fallback = b.clone();
            for i in 0..10 {
                let l = OrderLine{
                    exchange: String::from("binance"),
                    price: b.asks[i][0].parse::<f64>().unwrap(), 
                    amount: b.asks[i][1].parse::<f64>().unwrap()
                };
                o.asks.push(l);
            }
            for i in 0..10 {
                let l = OrderLine{
                    exchange: String::from("binance"),
                    price: b.bids[i][0].parse::<f64>().unwrap(), 
                    amount: b.bids[i][1].parse::<f64>().unwrap()
                };
                o.bids.push(l);
            }
            o.spread = o.asks[0].price - o.bids[0].price;
            self.book.write(o);
            tx.send(0).await.unwrap();
        }
    }
}