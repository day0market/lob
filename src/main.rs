mod connectors;
mod aggregators;

use std::io::Write;
use tokio::sync::mpsc::{Receiver,channel};
use connectors::OrderBookUpdate;
use serde_json;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::connectors::binance::BinanceOrderBookListener;
use crate::connectors::bitstamp::BitstampOrderBookListener;

async fn dump_data(mut receiver: Receiver<OrderBookUpdate>){
    let mut messages = Vec::with_capacity(20_000);
    while let Some(message) = receiver.recv().await {
        //println!("dump_received: {:?}", &message);
        messages.push(message);
        if messages.len() == 20_000{
            break
        }
    };

    //let dumped = serde_json::to_vec(&messages).unwrap();
    std::fs::write(
        "./updates.json",
        serde_json::to_string(&messages).unwrap(),
    ).unwrap()


}

#[tokio::main]
async fn main() {
    let (sender, receiver) = channel(3);
    let binance = BinanceOrderBookListener::new("BTC/USDT", 0);
    let bitstamp = BitstampOrderBookListener::new("BTC/USDT", 1);

    let t1 = {
        let sender = sender.clone();
        tokio::spawn(async move {
            binance.run(sender).await
        })
    };
    let t2 = {
        let sender = sender.clone();
        tokio::spawn(async move {
            bitstamp.run(sender).await
        })
    };
    let t3 = tokio::spawn(dump_data(receiver));

    t1.await.unwrap();
    t2.await.unwrap();
    t3.await.unwrap();
}
