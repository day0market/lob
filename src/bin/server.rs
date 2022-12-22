use lob::aggregation::aggregator::OrderBookAggregator;
use lob::aggregation::quote_merge::{IterativeMergeQuotes, MergeQuotes};
use lob::common::model::OrderBookUpdate;
use lob::connectors::binance::BinanceOrderBookListener;
use lob::connectors::bitstamp::BitstampOrderBookListener;
use lob::orderbook::OrderBookServer;
use lob::orderbook::{orderbook_aggregator_server::OrderbookAggregatorServer, Summary};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::watch::Receiver as WatchReceiver;
use tonic;
use tonic::transport::Server;

async fn order_book_aggregation<T: MergeQuotes>(
    mut receiver: Receiver<OrderBookUpdate>,
    mut sender: tokio::sync::watch::Sender<Summary>,
    mut order_book_aggregator: OrderBookAggregator<T>,
) {
    while let Some(message) = receiver.recv().await {
        if let Some(new_top) = order_book_aggregator.process(message) {
            if let Err(err) = sender.send(new_top) {
                println!("{:?}", err)
            }
        }
    }
}

async fn grpc_server(rx: WatchReceiver<Summary>, addr: SocketAddr) {
    let order_book_server = OrderBookServer::new(rx);
    let order_book_aggregator_server = OrderbookAggregatorServer::new(order_book_server);

    Server::builder()
        .add_service(order_book_aggregator_server)
        .serve(addr)
        .await
        .unwrap(); // TODO alex handle
}

#[tokio::main]
async fn main() {
    let top_book_depth = 10;
    let (exchange_order_book_sender, exchange_order_book_receiver) = channel(3);
    let (summary_sender, summary_receiver) = tokio::sync::watch::channel(Summary {
        spread: 0.0,
        bids: vec![],
        asks: vec![],
    });

    let binance = BinanceOrderBookListener::new("BTC/USDT", 0);
    let bitstamp = BitstampOrderBookListener::new("BTC/USDT", 1);

    let mut exchange_id_mapping = HashMap::new();
    exchange_id_mapping.insert(0, "binance".to_string());
    exchange_id_mapping.insert(1, "bitstamp".to_string());

    let exchanges_number = exchange_id_mapping.len();

    let book_merger = IterativeMergeQuotes::new(10, exchanges_number);

    let order_book_aggregator = OrderBookAggregator::new(
        book_merger,
        exchanges_number,
        top_book_depth,
        exchange_id_mapping,
    );

    let t1 = {
        let sender = exchange_order_book_sender.clone();
        tokio::spawn(async move { binance.run(sender).await })
    };
    let t2 = {
        let sender = exchange_order_book_sender.clone();
        tokio::spawn(async move { bitstamp.run(sender).await })
    };
    let t3 = tokio::spawn(async move {
        order_book_aggregation(
            exchange_order_book_receiver,
            summary_sender,
            order_book_aggregator,
        )
        .await
    });

    let addr = "0.0.0.0:50051".parse().unwrap();
    let t4 = tokio::spawn(async move { grpc_server(summary_receiver, addr).await });

    t1.await.unwrap();
    t2.await.unwrap();
    t3.await.unwrap();
    t4.await.unwrap();
}
