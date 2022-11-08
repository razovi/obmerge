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
        let item = stream.message().await.unwrap().unwrap();
        println!("received: {}", serde_json::to_string(&OrderBook::from_summary(item)).unwrap());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    let mut client = OrderbookAggregatorClient::connect("http://[::1]:50051").await.unwrap();
    streaming_ob(&mut client).await;
    Ok(())
}