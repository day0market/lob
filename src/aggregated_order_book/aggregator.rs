use crate::aggregated_order_book::quote_merge::MergeQuotes;
use crate::common::model::{AggregatedBookQuote, ExchangeQuote};
use crate::OrderBookUpdate;
use std::collections::HashMap;

pub struct OrderBookAggregator<T: MergeQuotes> {
    exchanges_bids: Vec<Vec<ExchangeQuote>>,
    exchanges_asks: Vec<Vec<ExchangeQuote>>,
    old_bid_book_top: Vec<AggregatedBookQuote>,
    old_ask_book_top: Vec<AggregatedBookQuote>,

    exchanges_number: usize,
    top_book_depth: usize,
    book_merger: T,
    exchanges_id_mapping: HashMap<usize, String>,
}

impl<T: MergeQuotes> OrderBookAggregator<T> {
    pub fn new(
        book_merger: T,
        exchanges_number: usize,
        top_book_depth: usize,
        exchanges_id_mapping: HashMap<usize, String>,
    ) -> Self {
        let mut exchanges_bids = Vec::with_capacity(exchanges_number);
        let mut exchanges_asks = Vec::with_capacity(exchanges_number);
        for _ in 0..exchanges_number {
            exchanges_bids.push(Vec::with_capacity(top_book_depth));
            exchanges_asks.push(Vec::with_capacity(top_book_depth));
        }

        let old_bid_book_top = Vec::with_capacity(top_book_depth.into());
        let old_ask_book_top = Vec::with_capacity(top_book_depth.into());

        Self {
            exchanges_bids,
            exchanges_asks,
            old_bid_book_top,
            old_ask_book_top,
            exchanges_number,
            top_book_depth,
            book_merger,
            exchanges_id_mapping,
        }
    }

    pub fn process(
        &mut self,
        order_book_update: OrderBookUpdate,
    ) -> Option<(Vec<AggregatedBookQuote>, Vec<AggregatedBookQuote>)> {
        let exchange_id = match order_book_update.exchange_id {
            Some(val) => val,
            None => {
                println!("exchange_id is empty. skip update");
                return None;
            }
        };

        let mut top_changed = false;

        self.exchanges_asks[exchange_id] = order_book_update.ask_changes; // TODO alex cut
        self.exchanges_bids[exchange_id] = order_book_update.bid_changes; // TODO alex cut

        if let Some(val) =
            self.book_merger
                .merge_quotes(&self.exchanges_bids, &self.old_bid_book_top, true)
        {
            top_changed = true;
            self.old_bid_book_top = val;
        }

        if let Some(val) =
            self.book_merger
                .merge_quotes(&self.exchanges_asks, &self.old_ask_book_top, false)
        {
            top_changed = true;
            self.old_ask_book_top = val;
        };

        if top_changed {
            return Some((self.old_bid_book_top.clone(), self.old_ask_book_top.clone()));
        };

        None
    }
}
