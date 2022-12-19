pub mod binance;
pub mod bitstamp;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ExchangeQuote{
    #[serde_as(as = "DisplayFromStr")]
    pub price: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub qty: f64,
}


#[derive(Deserialize, Debug, Serialize)]
pub struct OrderBookUpdate{
    pub exchange_id: Option<usize>,
    pub bid_changes: Vec<ExchangeQuote>,
    pub ask_changes: Vec<ExchangeQuote>,
}

pub trait Connector{

}