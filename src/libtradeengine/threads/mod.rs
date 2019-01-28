use engine::Order;
use engine::types::*;

pub mod matcher;
pub mod io;

pub enum MatcherThreadMessage {
  AddOrder(Order),
  Exit,
}

pub enum IoThreadMessage {
  AddOrderAck(OrderIdT),
  Exit,
}
