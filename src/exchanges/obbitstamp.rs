use std::net::TcpStream;
use crate::orderbook::{OrderLine, OrderBook};
use tungstenite::{connect, WebSocket, stream::MaybeTlsStream};
use url::Url;
use tokio::sync::mpsc::Sender;
use serde::{Serialize, Deserialize};
use tungstenite::Message;
use serde_json;

pub struct OBBitstamp{
    book: triple_buffer::Input<OrderBook>,
    pub status: Result<(), String>,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

#[derive(Serialize)]
struct Channel{
    channel: String
}

#[derive(Serialize)]
struct Subscribe{
    event: String,
    data: Channel
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct BitstampData{
    timestamp: String,
    microtimestamp: String,
    bids: Vec<[String; 3]>,
    asks: Vec<[String; 3]>,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct BitstampBook{
    data: BitstampData,
    channel: String,
    event: String
}

impl OBBitstamp {
    pub fn new(book: triple_buffer::Input<OrderBook>, pair: String) -> Self{
        let channel = Channel{channel: format!("detail_order_book_{}", pair)};
        let subscribe = Subscribe{
            event: String::from("bts:subscribe"),
            data: channel,
        };
        let url = format!("wss://ws.bitstamp.net");
        let r = connect(Url::parse(&url).unwrap());
        let mut socket = None;
        let ssocket;
        let response;
        let status = match r {
            Ok(x) => {
                (ssocket, response) = x;
                socket = Some(ssocket);
                println!("Connected to Bitstamp stream.");
                println!("HTTP status code: {}", response.status());

                let payload = serde_json::to_string(&subscribe).unwrap();
                socket.as_mut().unwrap().write_message(Message::Text(payload.into())).unwrap();
                let _ = socket.as_mut().unwrap().read_message().expect("Error reading message");
                Ok(())
            }
            Err(_) => {
                Err(String::from("Can't connect"))
            }
        };
        OBBitstamp {
            book,
            status,
            socket,
        }
    }
    pub async fn listen(&mut self, tx: Sender<usize>) {
        let mut fallback: BitstampBook = BitstampBook{data: BitstampData{timestamp: String::new(), microtimestamp: String::new(), bids: Vec::new(), asks: Vec::new()}, channel: String::new(), event: String::new()};
        loop{

            let msg = self.socket.as_mut().unwrap().read_message().expect("Error reading message");
            //println!("Received: {}", msg);
            let mut o: OrderBook = OrderBook{
                spread: 0.0,
                asks: Vec::new(),
                bids: Vec::new(),
            };

            let b: BitstampBook = serde_json::from_str(&msg.into_text().unwrap()[..]).unwrap_or(fallback);
            fallback = b.clone();
            for i in 0..10 {
                let l = OrderLine{
                    exchange: String::from("bitstamp"),
                    price: b.data.asks[i][0].parse::<f64>().unwrap(), 
                    amount: b.data.asks[i][1].parse::<f64>().unwrap()
                };
                o.asks.push(l);
            }
            for i in 0..10 {
                let l = OrderLine{
                    exchange: String::from("bitstamp"),
                    price: b.data.bids[i][0].parse::<f64>().unwrap(), 
                    amount: b.data.bids[i][1].parse::<f64>().unwrap()
                };
                o.bids.push(l);
            }
            o.spread = o.asks[0].price - o.bids[0].price;
            self.book.write(o);
            let _ = tx.send(0);
        }
    }
}