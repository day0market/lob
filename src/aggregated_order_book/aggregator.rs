use crate::aggregated_order_book::quote_merge::MergeQuotes;
use crate::common::model::{AggregatedBookQuote, ExchangeQuote};

pub struct ExchangesQuotesAggregator<T: MergeQuotes> {
    exchanges_quotes: Vec<Vec<ExchangeQuote>>,
    old_aggregated_book_top: Vec<AggregatedBookQuote>,
    total_exchanges: usize,
    top_book_depth: usize,
    book_merger: T,
    reverse_ordering: bool,
}

impl<T: MergeQuotes> ExchangesQuotesAggregator<T> {
    pub fn new(
        total_exchanges: usize,
        top_book_depth: usize,
        reverse_ordering: bool,
        book_merger: T,
    ) -> Self {
        let mut exchanges_quotes = Vec::with_capacity(total_exchanges);
        for _ in 0..total_exchanges {
            exchanges_quotes.push(Vec::with_capacity(top_book_depth))
        }

        let old_aggregated_book_top = Vec::with_capacity(top_book_depth.into());
        Self {
            exchanges_quotes,
            old_aggregated_book_top,
            total_exchanges,
            top_book_depth,
            book_merger,
            reverse_ordering,
        }
    }

    pub fn process(
        &mut self,
        exchange_quotes: Vec<ExchangeQuote>,
        exchange_id: usize,
    ) -> Option<Vec<AggregatedBookQuote>> {
        self.exchanges_quotes[exchange_id] = exchange_quotes;

        let new_top = self.book_merger.merge_quotes(
            &self.exchanges_quotes,
            &self.old_aggregated_book_top,
            self.reverse_ordering,
        );
        match new_top {
            Some(val) => {
                self.old_aggregated_book_top = val.clone();
                Some(val)
            }
            None => None,
        }
    }
}
