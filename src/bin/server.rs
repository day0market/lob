use clap::Parser;
use lob::aggregation::aggregator::OrderBookAggregator;
use lob::aggregation::quote_merge::{IterativeMergeQuotes, MergeQuotes};
use lob::common::model::OrderBookUpdate;
use lob::connectors::binance::BinanceOrderBookListener;
use lob::connectors::bitstamp::BitstampOrderBookListener;
use lob::orderbook::OrderbookAggregatorPublisher;
use lob::orderbook::{orderbook_aggregator_server::OrderbookAggregatorServer, Summary};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::watch::Receiver as WatchReceiver;
use tonic;
use tonic::transport::Server;
use tracing::{error, info};
use tracing_subscriber;

async fn order_book_aggregation<T: MergeQuotes>(
    mut receiver: Receiver<OrderBookUpdate>,
    sender: tokio::sync::watch::Sender<Summary>,
    mut order_book_aggregator: OrderBookAggregator<T>,
) {
    while let Some(message) = receiver.recv().await {
        info!("received new order book update: {:?}", &message);
        if let Some(new_top) = order_book_aggregator.process(message) {
            info!("book top updated: {:?}", &new_top);
            if let Err(err) = sender.send(new_top) {
                error!("failed to send new top. err={:?}", err)
            }
        }
    }
}

async fn grpc_server(rx: WatchReceiver<Summary>, addr: SocketAddr) {
    let publisher = OrderbookAggregatorPublisher::new(rx);
    let server = OrderbookAggregatorServer::new(publisher);

    Server::builder()
        .add_service(server)
        .serve(addr)
        .await
        .unwrap();
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    //#[clap(short, long)]
    //symbol: String,
    #[clap(short, long, default_value_t = 10)]
    top_book_depth: usize,
    #[clap(short, long, default_value_t = 50051)]
    port: usize,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let symbol = "BTC/USDT".to_string();

    let (exchange_order_book_sender, exchange_order_book_receiver) = channel(3);

    let (summary_sender, summary_receiver) = tokio::sync::watch::channel(Summary {
        spread: 0.0,
        bids: vec![],
        asks: vec![],
    });

    let binance = BinanceOrderBookListener::new(&symbol, 0);
    let bitstamp = BitstampOrderBookListener::new(&symbol, 1);

    let mut exchange_id_mapping = HashMap::new();
    exchange_id_mapping.insert(0, "binance".to_string());
    exchange_id_mapping.insert(1, "bitstamp".to_string());

    let exchanges_number = exchange_id_mapping.len();

    let quotes_merger = IterativeMergeQuotes::new(args.top_book_depth, exchanges_number);

    let order_book_aggregator = OrderBookAggregator::new(
        quotes_merger,
        exchanges_number,
        args.top_book_depth,
        exchange_id_mapping,
    );

    let order_book_aggregation = tokio::spawn(async move {
        order_book_aggregation(
            exchange_order_book_receiver,
            summary_sender,
            order_book_aggregator,
        )
        .await
    });

    let binance_order_book_handler = {
        let sender = exchange_order_book_sender.clone();
        tokio::spawn(async move { binance.run(sender).await })
    };
    let bitstamp_order_book_handler = {
        let sender = exchange_order_book_sender.clone();
        tokio::spawn(async move { bitstamp.run(sender).await })
    };

    let addr = format!("0.0.0.0:{}", args.port).parse().unwrap();
    let grpc_server = tokio::spawn(async move { grpc_server(summary_receiver, addr).await });

    binance_order_book_handler.await.unwrap();
    bitstamp_order_book_handler.await.unwrap();
    order_book_aggregation.await.unwrap();
    grpc_server.await.unwrap();
}
