use engine::types::*;
use engine::Order;

pub mod io;
pub mod matcher;

pub enum MatcherThreadMessage {
    AddOrder(Order),
    Exit,
}

pub enum IoThreadMessage {
    AddOrderAck(OrderIdT),
    Exit,
}
