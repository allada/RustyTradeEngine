#[cfg(test)]
use std::cmp;
use std::cmp::Ordering;

#[cfg(test)]
use super::types::*;
use super::Order;

impl Trade {
    pub fn execute(taker: Order, maker: Order) -> (Trade, Option<Order>) {
        let new_order = match taker.qty().cmp(maker.qty()) {
            Ordering::Less => Some(Order::copy_with_new_qty(
                &maker,
                *maker.qty() - *taker.qty(),
            )),
            Ordering::Greater => Some(Order::copy_with_new_qty(
                &taker,
                *taker.qty() - *maker.qty(),
            )),
            _ => None,
        };

        (Trade { taker, maker }, new_order)
    }

    #[cfg(test)]
    pub fn taker(&self) -> &Order {
        &self.taker
    }

    #[cfg(test)]
    pub fn maker(&self) -> &Order {
        &self.maker
    }

    #[cfg(test)]
    pub fn price(&self) -> &PriceT {
        self.maker.price()
    }

    #[cfg(test)]
    pub fn qty(&self) -> &QtyT {
        cmp::min(self.taker.qty(), self.maker.qty())
    }
}

#[derive(Debug)]
pub struct Trade {
    taker: Order,
    maker: Order,
}
