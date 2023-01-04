use clap::Parser;
use futures_util::StreamExt;
use lob::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient;
use lob::orderbook::Empty;
use tonic::transport::Endpoint;
use tonic::Request;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long, default_value_t = 50051)]
    port: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let url = format!("https://127.0.0.1:{}", args.port);
    let addr = Endpoint::from_shared(url)?;

    let mut client = OrderbookAggregatorClient::connect(addr).await?;
    let request = Request::new(Empty {});
    let mut response = client.book_summary(request).await.unwrap().into_inner();

    while let Some(row) = response.next().await {
        println!("new: {:#?}", &row);
    }

    Ok(())
}
