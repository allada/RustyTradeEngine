use std::collections::BinaryHeap;
use std::collections::binary_heap::PeekMut;
use std::vec::Vec;
use std::cmp::Ordering;

use super::types::*;
use super::Order;
use super::Trade;

impl Ledger {
  pub fn new() -> Ledger {
    Ledger {
      buy_ledger: BinaryHeap::new(),
      sell_ledger: BinaryHeap::new(),
    }
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
      if *order.order_type() == OrderTypeT::MARKET || (*peek).price().cmp(order.price()) != cmp_type {
        return Some(PeekMut::pop(peek));
      }
    }
    return None;
  }

  pub fn add_order(&mut self, order: Order) -> Vec<Trade> {
    let mut new_trades = Vec::new();
    let origin_order_id = *order.id();

    let mut maybe_order = Some(order);
    while let Some(taker) = maybe_order.take() {
      if *taker.id() != origin_order_id {
        self.stash_order(taker);
        break;
      }
      let maybe_maker = self.get_matched_order(&taker);
      if let Some(maker) = maybe_maker {
        // Take order off tree and consume order since we found a match.
        let trade_tuple = Trade::execute(taker, maker);
        new_trades.push(trade_tuple.0);
        maybe_order = trade_tuple.1;
        // TODO Do something with trade.
      } else {
        if *taker.order_type() == OrderTypeT::MARKET {
          // Ledger is empty and it's a market order, so just cancel.
          break;
        }
        // Put onto ledger, now is a maker.
        self.stash_order(taker);
        break;
      }
    }
    new_trades
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

pub struct Ledger {
  buy_ledger: BinaryHeap<Order>,
  sell_ledger: BinaryHeap<Order>,
}