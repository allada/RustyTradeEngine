
extern crate uuid;

pub type OrderIdT = uuid::Bytes;
pub type PriceT = u64;
pub type QtyT = u64;

pub use proto::OrderTypeT;
pub use proto::SideT;
