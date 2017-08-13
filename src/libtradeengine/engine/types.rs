
pub type OrderIdT = u64;
pub type PriceT = u64;
pub type QtyT = u64;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SideT {
  BUY,
  SELL,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OrderTypeT {
  MARKET,
  LIMIT,
}
