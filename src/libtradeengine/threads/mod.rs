use engine::Order;
use engine::types::*;

pub mod ui;
pub mod io;

pub enum UiThreadMessage {
  AddOrder(Order),
  Exit,
}

pub enum IoThreadMessage {
  AddOrderAck(OrderIdT),
  Exit,
}
