use std::cmp::Ordering;

use std::collections::binary_heap::PeekMut;
use std::collections::BinaryHeap;
use std::vec::Vec;

use super::types::*;
use super::Order;
use super::Trade;
use engine::AddOrderResults;
use engine::AddOrderError;
use engine::LedgerMutation;

impl Ledger {
    pub fn add_order(&mut self, order: Order) -> AddOrderResults {
        let mut ledger_mutations: Vec<LedgerMutation> = Vec::new();
        let origin_order_id = *order.id();

        // let mut new_maker_order: Option<Order> = None;
        let mut maybe_order = Some(order);
        while let Some(taker) = maybe_order.take() {
            if *taker.id() != origin_order_id {
                // Put remaining order that is not ours back on the ledger.
                // Our order seems to have been fully filled.
                ledger_mutations.push(LedgerMutation::AddedMakerOrder(taker.clone()));
                self.stash_order(taker);
                break;
            }
            let maybe_maker = self.get_matched_order(&taker);
            if let Some(maker) = maybe_maker {
                // Take order off tree and consume order since we found a match.
                let (new_trade, maybe_remaining_order) = Trade::execute(taker, maker);
                ledger_mutations.push(LedgerMutation::TradeExecuted(new_trade));
                maybe_order = maybe_remaining_order;
            // TODO Do something with trade.
            } else {
                if *taker.order_type() == OrderTypeT::MARKET {
                    self.revert_ledger_mutations(ledger_mutations);
                    return Err(AddOrderError::NotEnoughOrdersToFillMarketOrder);
                }
                // Put onto ledger, now is a maker.
                ledger_mutations.push(LedgerMutation::AddedMakerOrder(taker.clone()));
                self.stash_order(taker);
                break;
            }
        }
        Ok(ledger_mutations)
    }

    fn revert_ledger_mutations(&mut self, _ledger_mutations: Vec<LedgerMutation>) {
        // let mut last_maker_price = if let Some(last_trade) = trades.last() {
        //     *last_trade.maker().price()
        // } else {
        //     return;
        // };
        // for trade in trades {
        //     debug_assert!(if *trade.maker().side() == SideT::SELL {
        //         *trade.maker().price() >= last_maker_price
        //     } else {
        //         *trade.maker().price() <= last_maker_price
        //     });
        //     let mut order = *trade.maker();
        //     self.stash_order(order);
        // }
    }

    fn stash_order(&mut self, order: Order) {
        debug_assert!(*order.order_type() != OrderTypeT::MARKET);
        if *order.side() == SideT::BUY {
            self.buy_ledger.push(order);
        } else {
            self.sell_ledger.push(order);
        }
    }

    fn get_matched_order(&mut self, order: &Order) -> Option<Order> {
        // NOTE: cmpType is always inversed to be not equal too, so it adds Eq into it.
        let (maybe_peek, cmp_type) = if *order.side() == SideT::SELL {
            (self.buy_ledger.peek_mut(), Ordering::Less)
        } else {
            (self.sell_ledger.peek_mut(), Ordering::Greater)
        };
        if let Some(peek) = maybe_peek {
            if *order.order_type() == OrderTypeT::MARKET
                || (*peek).price().cmp(order.price()) != cmp_type
            {
                return Some(PeekMut::pop(peek));
            }
        }
        None
    }

    #[cfg(test)]
    pub fn buy_ledger_for_test(&self) -> &BinaryHeap<Order> {
        &self.buy_ledger
    }

    #[cfg(test)]
    pub fn sell_ledger_for_test(&self) -> &BinaryHeap<Order> {
        &self.sell_ledger
    }
}

#[derive(Default)]
pub struct Ledger {
    buy_ledger: BinaryHeap<Order>,
    sell_ledger: BinaryHeap<Order>,
}

#[derive(Debug)]
pub enum NewOrderError {
    MarketOrderWithEmptyLedger,
}
