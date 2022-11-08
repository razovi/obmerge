use obmerge::orderbook::OrderBook;
use obmerge::proto::order_book::Empty;
use obmerge::proto::order_book::orderbook_aggregator_client::OrderbookAggregatorClient; 
use serde_json;
use tonic::Request;
use tonic::transport::Channel;

async fn streaming_ob(client: &mut OrderbookAggregatorClient<Channel>) {
    let mut stream = client
        .book_summary(Request::new(Empty{}))
        .await
        .unwrap()
        .into_inner();

    loop{
        let item = stream.message().await;
        match item {
            Ok(x) => {
                if let Some(y) = x{
                    println!("received: {}", serde_json::to_string(&OrderBook::from_summary(y)).unwrap());
                }
            }
            Err(_) => {
                println!("The server isn't running!...exiting...");
                break;
            }
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let addr = "http://[::1]:50051";
    let client = OrderbookAggregatorClient::connect(addr).await;
    match client {
        Ok(mut x) => {
            println!("Listening on {}", addr);
            streaming_ob(&mut x).await;
        }
        Err(_) => {
            println!("The server isn't running!...exiting...");
        }
    }
    Ok(())
}