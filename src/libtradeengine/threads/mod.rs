use engine;
use std::vec::Vec;

pub mod io;
pub mod matcher;

pub enum MatcherThreadMessage {
    AddOrder(engine::Order),
    Exit,
}

pub enum IoThreadMessage {
    ProcessRawData(Vec<u8>),
    AddOrderAck(engine::OrderIdT),
    Exit,
}
