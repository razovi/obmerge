use std::net::TcpStream;
use crate::orderbook::{OrderLine, OrderBook};
use tungstenite::{connect, WebSocket, stream::MaybeTlsStream, protocol::Message};
use url::Url;
use tokio::sync::mpsc::Sender;
use serde_json;
use serde::Deserialize;


pub struct OBBinance{
    book: triple_buffer::Input<OrderBook>,
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
    pub fn new(book: triple_buffer::Input<OrderBook>, pair: String) -> Result<Self, &'static str>{
        let url = format!("wss://stream.binance.com:9443/ws/{}@depth20@100ms/ws", pair);
        let r = connect(Url::parse(&url).unwrap());
        match r {
            Ok(x) => {
                let (socket, _response) = x;
                Ok(OBBinance{
                    book,
                    socket: Some(socket),
                })
            }
            Err(_) => {
                Err("Can't connect!")
            }
        }
       
    }
    pub async fn listen(&mut self, tx: Sender<usize>) {
        let mut fallback = BinanceBook{lastUpdateId: 0, bids: Vec::new(), asks: Vec::new()};
        let mut frame = Vec::new();
        loop{
            // send PONG frame after receiving a PING
            if !frame.is_empty() {
                self.socket.as_mut().unwrap().write_message(Message::Pong(frame.clone())).unwrap();
                frame.clear();
            }

            let mut o: OrderBook = OrderBook{
                spread: 0.0,
                asks: Vec::new(),
                bids: Vec::new(),
            };

            let msg = self.socket.as_mut().unwrap().read_message().expect("Error reading message");
            match msg {
                Message::Ping(x) => {
                    frame = x;
                }
                Message::Text(x) => {
                    let b: BinanceBook = serde_json::from_str(&x[..]).unwrap_or(fallback);
                    fallback = b.clone();
                    // get 10 asks
                    for i in 0..10 {
                        let l = OrderLine{
                            exchange: String::from("binance"),
                            price: b.asks[i][0].parse::<f64>().unwrap(), 
                            amount: b.asks[i][1].parse::<f64>().unwrap()
                        };
                        o.asks.push(l);
                    }
                    // get 10 bids
                    for i in 0..10 {
                        let l = OrderLine{
                            exchange: String::from("binance"),
                            price: b.bids[i][0].parse::<f64>().unwrap(), 
                            amount: b.bids[i][1].parse::<f64>().unwrap()
                        };
                        o.bids.push(l);
                    }
                    // get spread
                    o.spread = o.asks[0].price - o.bids[0].price;
                    // update orderbook and send update message
                    self.book.write(o);
                    let _ = tx.send(0).await;
                }
                _ => {
                    panic!("Unexpected Binance Message");
                }
            }
        }
    }
}