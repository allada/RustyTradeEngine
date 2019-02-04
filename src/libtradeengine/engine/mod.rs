pub mod ledger;
pub mod order;
pub mod trade;
pub mod types;

pub use self::ledger::Ledger;
pub use self::order::Order;
pub use self::trade::Trade;
pub use self::types::*;

/// Action to process on matcher thread.
#[derive(Debug)]
pub enum MatcherAction {
    AddOrder(Order),
    // CancelOrder(engine::OrderIdT),
    Shutdown,
}

#[derive(Debug, PartialEq)]
pub enum AddOrderError {
    NotEnoughOrdersToFillMarketOrder,
}

#[cfg(test)]
impl LedgerMutation {
    fn is_trade(&self) -> bool {
      return match self {
        LedgerMutation::TradeExecuted(_) => true,
        _ => false,
      }
    }

    fn as_trade<'a>(&'a self) -> &'a Trade {
      return match self {
        LedgerMutation::TradeExecuted(trade) => trade,
        _ => panic!("Not a trade."),
      };
    }

    fn is_add_maker(&self) -> bool {
      return match self {
        LedgerMutation::AddedMakerOrder(_) => true,
        _ => false,
      }
    }

    fn as_add_maker<'a>(&'a self) -> &'a Order {
      return match self {
        LedgerMutation::AddedMakerOrder(order) => order,
        _ => panic!("Not an order."),
      };
    }
}

#[derive(Debug)]
pub enum LedgerMutation {
    AddedMakerOrder(Order),
    TradeExecuted(Trade),
    // RemoveMakerOrder(Order),
}

pub type AddOrderResults = Result<Vec<LedgerMutation>, AddOrderError>;

#[derive(Debug)]
pub enum MatcherActionResult {
    AddOrder(AddOrderResults),
    // CancelOrder(Result<LedgerMutation, CancelOrderError>),
    ShutdownAck,
}

#[cfg(test)]
mod tests;
