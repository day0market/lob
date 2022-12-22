mod aggregated_order_book;
mod common;
mod connectors;

use crate::aggregated_order_book::aggregator::OrderBookAggregator;
use crate::aggregated_order_book::quote_merge::{IterativeMergeQuotes, MergeQuotes};
use crate::common::model::AggregatedBookQuote;
use crate::connectors::binance::BinanceOrderBookListener;
use crate::connectors::bitstamp::BitstampOrderBookListener;
use common::model::OrderBookUpdate;
use serde_json;
use std::collections::HashMap;
use std::io::Write;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{channel, Receiver};

async fn order_book_aggregation<T: MergeQuotes>(
    mut receiver: Receiver<OrderBookUpdate>,
    mut order_book_aggregator: OrderBookAggregator<T>,
) {
    while let Some(message) = receiver.recv().await {
        let exch_id = message.exchange_id.unwrap_or_else(|| 0);
        if let Some(new_top) = order_book_aggregator.process(message) {
            if exch_id == 1 {
                println!("{:#?}", new_top)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let top_book_depth = 10;
    let (sender, receiver) = channel(3);
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
        let sender = sender.clone();
        tokio::spawn(async move { binance.run(sender).await })
    };
    let t2 = {
        let sender = sender.clone();
        tokio::spawn(async move { bitstamp.run(sender).await })
    };
    let t3 =
        tokio::spawn(async move { order_book_aggregation(receiver, order_book_aggregator).await });

    t1.await.unwrap();
    t2.await.unwrap();
    t3.await.unwrap();
}
