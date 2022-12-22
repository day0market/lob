
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
    },
};
use serde::{Deserialize};
use serde_json;
use tokio::sync::mpsc::Sender;
use flate2::read::GzDecoder;
use futures::{StreamExt, SinkExt};
use crate::common::model::{OrderBookUpdate, ExchangeQuote};
use std::io::Read;




pub struct BinanceOrderBookListener{
    exchange_symbol: String,
    exchange_id: usize
}


//{
//   "lastUpdateId": 160,  // Last update ID
//   "bids": [             // Bids to be updated
//     [
//       "0.0024",         // Price level to be updated
//       "10"              // Quantity
//     ]
//   ],
//   "asks": [             // Asks to be updated
//     [
//       "0.0026",         // Price level to be updated
//       "100"             // Quantity
//     ]
//   ]
// }

#[derive(Deserialize, Debug)]
struct BinanceOrderBookUpdate{
    bids: Vec<ExchangeQuote>,
    asks: Vec<ExchangeQuote>,
}

impl Into<OrderBookUpdate> for BinanceOrderBookUpdate {
    fn into(self) -> OrderBookUpdate {
        OrderBookUpdate{
            bid_changes: self.bids,
            ask_changes: self.asks,
            exchange_id: None,
        }
    }
}

impl BinanceOrderBookListener{
    pub fn new(pair: &str, exchange_id: usize) -> Self{
        let exchange_symbol = pair.replace("/", "");
        Self{
            exchange_symbol,
            exchange_id
        }
    }
    pub async fn run(&self, pub_chan: Sender<OrderBookUpdate>){
        let subscription_url = format!("wss://stream.binance.com:443/ws/{}@depth{}@100ms", &self.exchange_symbol.to_lowercase(), 20);


        'reconnection_loop: loop {
            let (mut stream, _) = connect_async(subscription_url.clone()).await.unwrap();

            loop {
                let rcv = stream.next().await;
                if rcv.is_none(){
                    continue
                }
                let raw_msg = match rcv.unwrap(){
                    Ok(val) => val,
                    Err(err) => {
                        // TODO Alex: handle errors
                        println!("err: {:?}", err);
                        stream.close(Some(CloseFrame {
                            code: CloseCode::Normal,
                            reason: "Client requested connection close.".into(),
                        })).await;
                        continue 'reconnection_loop
                    }
                };


                let binance_message = match raw_msg {
                    Message::Text(raw_msg) => {
                        let value: BinanceOrderBookUpdate = serde_json::from_str(&raw_msg).unwrap();
                        value
                    },
                    Message::Binary(raw_msg) => {
                        let mut d = GzDecoder::new(&*raw_msg);
                        let mut s = String::new();
                        d.read_to_string(&mut s).unwrap();
                        let value: BinanceOrderBookUpdate = serde_json::from_str(&s).unwrap();
                        value
                    },
                    Message::Ping(_) => {
                         if let Err(err) = stream.send(Message::Pong("pong".into())).await{

                         };
                        continue
                    },
                    other =>{
                        println!("received unexpected: {:?}", other);
                        continue
                    }
                };


                let mut order_book_update: OrderBookUpdate = binance_message.into();
                order_book_update.exchange_id = Some(self.exchange_id);
                if let Err(err) = pub_chan.send(order_book_update).await{
                    println!("{}", err);
                    return;
                }


            }
        }

    }
}

