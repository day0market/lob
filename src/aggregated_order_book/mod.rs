mod aggregator;
mod quote_merge;

#[cfg(test)]
mod tests {
    use crate::aggregated_order_book::quote_merge::{
        IterativeMergeQuotes, MergeQuotes, VecSortMergeQuotes,
    };
    use crate::common::model::ExchangeQuote;
    use std::cmp::Ordering;

    fn exchanges_quotes_asks_fixture() -> Vec<Vec<ExchangeQuote>> {
        let changes1 = vec![
            ExchangeQuote {
                price: 1.0,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 1.2,
                qty: 2.2,
            },
            ExchangeQuote {
                price: 1.25,
                qty: 2.2,
            },
            ExchangeQuote {
                price: 1.35,
                qty: 2.2,
            },
            ExchangeQuote {
                price: 1.45,
                qty: 2.2,
            },
            ExchangeQuote {
                price: 1.65,
                qty: 2.2,
            },
            ExchangeQuote {
                price: 1.68,
                qty: 2.2,
            },
            ExchangeQuote {
                price: 1.69,
                qty: 2.2,
            },
            ExchangeQuote {
                price: 1.75,
                qty: 2.25,
            },
            ExchangeQuote {
                price: 1.83,
                qty: 0.25,
            },
            ExchangeQuote {
                price: 1.93,
                qty: 0.25,
            },
        ];

        let changes2 = vec![
            ExchangeQuote {
                price: 1.1,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 1.2,
                qty: 2.4,
            },
            ExchangeQuote {
                price: 1.25,
                qty: 1.9,
            },
            ExchangeQuote {
                price: 1.36,
                qty: 2.2,
            },
            ExchangeQuote {
                price: 1.46,
                qty: 2.2,
            },
            ExchangeQuote {
                price: 1.55,
                qty: 5.0,
            },
            ExchangeQuote {
                price: 1.65,
                qty: 2.2,
            },
            ExchangeQuote {
                price: 1.68,
                qty: 2.2,
            },
            ExchangeQuote {
                price: 1.69,
                qty: 2.2,
            },
            ExchangeQuote {
                price: 1.74,
                qty: 2.25,
            },
            ExchangeQuote {
                price: 1.84,
                qty: 0.25,
            },
            ExchangeQuote {
                price: 1.93,
                qty: 0.25,
            },
        ];

        vec![changes1, changes2]
    }

    fn exchanges_quotes_bids_fixture() -> Vec<Vec<ExchangeQuote>> {
        let changes1 = vec![
            ExchangeQuote {
                price: 1.0,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 0.98,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 0.975,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 0.97,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 0.94,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 0.92,
                qty: 3.0,
            },
            ExchangeQuote {
                price: 0.915,
                qty: 5.0,
            },
            ExchangeQuote {
                price: 0.91,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 0.9,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 0.8,
                qty: 12.0,
            },
            ExchangeQuote {
                price: 0.7,
                qty: 12.0,
            },
            ExchangeQuote {
                price: 0.4,
                qty: 42.0,
            },
        ];

        let changes2 = vec![
            ExchangeQuote {
                price: 1.1,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 0.98,
                qty: 5.0,
            },
            ExchangeQuote {
                price: 0.97,
                qty: 3.0,
            },
            ExchangeQuote {
                price: 0.96,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 0.94,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 0.925,
                qty: 5.0,
            },
            ExchangeQuote {
                price: 0.912,
                qty: 5.0,
            },
            ExchangeQuote {
                price: 0.911,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 0.9,
                qty: 2.0,
            },
            ExchangeQuote {
                price: 0.85,
                qty: 12.0,
            },
            ExchangeQuote {
                price: 0.71,
                qty: 12.0,
            },
            ExchangeQuote {
                price: 0.46,
                qty: 42.0,
            },
        ];

        vec![changes1, changes2]
    }

    #[test]
    fn iterative_merge() {
        let top_book_depth = 10;
        let order_books = exchanges_quotes_asks_fixture();
        let n_exchanges = order_books.len();
        let old_top = vec![];
        let mut merger = IterativeMergeQuotes::new(top_book_depth, n_exchanges);
        let top_book = merger.merge_quotes(&order_books, &old_top, false);
        assert!(top_book.is_some());

        let top_book = top_book.unwrap();
        assert_eq!(top_book.len(), top_book_depth);

        let mut best_quote = &top_book[0];
        for quote in &top_book[1..] {
            assert_eq!(
                quote.cmp(best_quote),
                Ordering::Greater,
                "{:?} {:?}",
                quote,
                best_quote
            );
            best_quote = quote;
        }

        let top_book_after_same_quotes = merger.merge_quotes(&order_books, &top_book, false);
        assert!(top_book_after_same_quotes.is_none());
    }

    #[test]
    fn iterative_merge_inverse_ordering() {
        let top_book_depth = 10;
        let order_books = exchanges_quotes_bids_fixture();
        let n_exchanges = order_books.len();
        let old_top = vec![];
        let mut merger = IterativeMergeQuotes::new(top_book_depth, n_exchanges);
        let top_book = merger.merge_quotes(&order_books, &old_top, true);
        assert!(top_book.is_some());

        let top_book = top_book.unwrap();
        assert_eq!(top_book.len(), top_book_depth);
        println!("{:#?}", &top_book);

        let mut best_quote = &top_book[0];
        for quote in &top_book[1..] {
            assert_eq!(
                quote.cmp(best_quote),
                Ordering::Less,
                "{:?} {:?}",
                quote,
                best_quote
            );
            best_quote = quote;
        }

        let top_book_after_same_quotes = merger.merge_quotes(&order_books, &top_book, true);
        assert!(top_book_after_same_quotes.is_none());
    }

    #[test]
    fn vec_sort_merge() {
        let top_book_depth = 10;
        let order_books = exchanges_quotes_asks_fixture();
        let n_exchanges = order_books.len();
        let old_top = vec![];
        let mut merger = VecSortMergeQuotes::new(top_book_depth, n_exchanges);
        let top_book = merger.merge_quotes(&order_books, &old_top, true);
        assert!(top_book.is_some());

        let top_book = top_book.unwrap();
        assert_eq!(top_book.len(), top_book_depth);

        let mut best_quote = &top_book[0];
        for quote in &top_book[1..] {
            assert_eq!(
                quote.cmp(best_quote),
                Ordering::Greater,
                "{:?} {:?}",
                quote,
                best_quote
            );
            best_quote = quote;
        }

        let top_book_after_same_quotes = merger.merge_quotes(&order_books, &top_book, false);
        assert!(top_book_after_same_quotes.is_none());
    }

    #[test]
    fn vec_sort_merge_inverse_ordering() {
        let top_book_depth = 10;
        let order_books = exchanges_quotes_bids_fixture();
        let n_exchanges = order_books.len();
        let old_top = vec![];
        let mut merger = VecSortMergeQuotes::new(top_book_depth, n_exchanges);
        let top_book = merger.merge_quotes(&order_books, &old_top, true);
        assert!(top_book.is_some());

        let top_book = top_book.unwrap();
        assert_eq!(top_book.len(), top_book_depth);
        println!("{:#?}", &top_book);

        let mut best_quote = &top_book[0];
        for quote in &top_book[1..] {
            assert_eq!(
                quote.cmp(best_quote),
                Ordering::Less,
                "{:?} {:?}",
                quote,
                best_quote
            );
            best_quote = quote;
        }

        let top_book_after_same_quotes = merger.merge_quotes(&order_books, &top_book, true);
        assert!(top_book_after_same_quotes.is_none());
    }
}
