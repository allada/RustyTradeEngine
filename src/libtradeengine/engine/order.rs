use std::cmp::Ordering;

use super::types::*;

impl Order {
  pub fn new(id: OrderIdT, price: PriceT, qty: QtyT, side: SideT, order_type: OrderTypeT) -> Order {
    Order {
      id,
      price,
      qty,
      side,
      order_type,
    }
  }

  pub fn copy_with_new_qty(other: &Order, qty: QtyT) -> Order {
    Order {
      id: other.id,
      price: other.price,
      qty,
      side: other.side,
      order_type: other.order_type,
    }
  }

  pub fn id(&self) -> &OrderIdT {
    &self.id
  }

  pub fn price(&self) -> &PriceT {
    &self.price
  }

  pub fn qty(&self) -> &QtyT {
    &self.qty
  }

  pub fn side(&self) -> &SideT {
    &self.side
  }

  pub fn order_type(&self) -> &OrderTypeT {
    &self.order_type
  }
}

impl PartialEq for Order {
  fn eq(&self, other: &Order) -> bool {
    self.id == other.id
  }
}

impl Eq for Order {}

impl PartialOrd for Order {
  fn partial_cmp(&self, other: &Order) -> Option<Ordering> {
    Some(self.cmp(other))
  }
  fn lt(&self, other: &Order) -> bool {
    self.cmp(other) == Ordering::Less
  }
  fn le(&self, other: &Order) -> bool {
    self.cmp(other) != Ordering::Greater
  }
  fn gt(&self, other: &Order) -> bool {
    self.cmp(other) == Ordering::Greater
  }
  fn ge(&self, other: &Order) -> bool {
    self.cmp(other) != Ordering::Less
  }
}
impl Ord for Order {
  fn cmp(&self, other: &Order) -> Ordering {
    let (a, b) = if *self.side() == SideT::SELL {
      (other, self)
    } else {
      (self, other)
    };
    let cmp = a.price.cmp(&b.price);
    if cmp != Ordering::Equal {
      return cmp;
    }
    a.id.cmp(&b.id)
  }
}

#[derive(Debug)]
pub struct Order {
  id: OrderIdT,
  price: PriceT,
  qty: QtyT,
  side: SideT,
  order_type: OrderTypeT,
}
