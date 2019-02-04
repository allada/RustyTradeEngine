use std::mem::size_of;

use super::types::*;

use super::AddOrderError;
use super::Ledger;
use super::Order;
use super::Trade;
use super::AddOrderResults;
use super::LedgerMutation;

fn to_order_id(order_id: &u64) -> OrderIdT {
    let mut result: OrderIdT = [0; size_of::<OrderIdT>()];
    let bytes = order_id.to_le_bytes();
    for i in 0..bytes.len() {
        result[i] = bytes[i];
    }
    return result;
}

fn from_order_id(order_id: &OrderIdT) -> u64 {
    let mut result: [u8; 8] = [0; 8];
    let slice = &order_id[0..8];
    assert_eq!(order_id[8..], [0; size_of::<OrderIdT>() - 8]);
    result.clone_from_slice(slice);
    return u64::from_le_bytes(result);
}

impl DummyOrder {
    pub fn new(
        id: u64,
        price: PriceT,
        qty: QtyT,
        side: SideT,
        order_type: OrderTypeT,
    ) -> DummyOrder {
        DummyOrder {
            id: to_order_id(&id),
            price: price,
            qty: qty,
            side: side,
            order_type: order_type,
        }
    }

    pub fn from_order(order: &Order) -> DummyOrder {
        DummyOrder::new(from_order_id(order.id()), *order.price(), *order.qty(), *order.side(), *order.order_type())
    }

    pub fn to_order(&self) -> Order {
        Order::new(self.id, self.price, self.qty, self.side, self.order_type)
    }

    pub fn verify(&self, order: &Order, ignore_qty: bool) {
        assert_eq!(*order.id(), self.id, "Order ids do not match.");
        assert_eq!(*order.price(), self.price, "Order prices dont match.");
        assert_eq!(*order.side(), self.side, "Order sides do not match.");
        assert_eq!(
            *order.order_type(),
            self.order_type,
            "Order types do not match."
        );
        if !ignore_qty {
            assert_eq!(*order.qty(), self.qty);
        }
    }

    // pub fn id(&self) -> &OrderIdT {
    //   &self.id
    // }

    pub fn price(&self) -> &PriceT {
        &self.price
    }

    pub fn qty(&self) -> &QtyT {
        &self.qty
    }

    // pub fn side(&self) -> &SideT {
    //   &self.side
    // }

    // pub fn order_type(&self) -> &OrderTypeT {
    //   &self.order_type
    // }
}

#[derive(Clone)]
struct DummyOrder {
    id: OrderIdT,
    price: PriceT,
    qty: QtyT,
    side: SideT,
    order_type: OrderTypeT,
}

mod ledger {
    use super::*;

    fn verify_trade(
        trade: &Trade,
        taker: &DummyOrder,
        maker: &DummyOrder,
        trade_price: &PriceT,
        trade_qty: &QtyT,
    ) {
        let trade_taker = trade.taker();
        let trade_maker = trade.maker();
        taker.verify(trade_taker, true);
        maker.verify(trade_maker, true);
        assert_eq!(*trade.qty(), *trade_qty, "Trade qty does not match.");
        assert_eq!(
            *trade.price(),
            *trade_price,
            "Trade price does not match the input param."
        );
        assert_eq!(
            *maker.price(),
            *trade_price,
            "Input param price is expected to be maker price."
        );
        assert!(
            *taker.qty() == *trade_qty || *maker.qty() == *trade_qty,
            "Expected one of the orders to be of trade_qty {}",
            *trade_qty
        );
    }

    fn verify_mutations(results: &AddOrderResults, expected_mutations: &Vec<LedgerMutation>) {
        assert!(results.is_ok());
        let mutations = results.as_ref().unwrap();
        assert_eq!(mutations.len(), expected_mutations.len());

        for i in 0..expected_mutations.len() {
            let real_mutation = &mutations[i];
            let expected_mutation = &expected_mutations[i];
            if expected_mutation.is_add_maker() {
                assert!(real_mutation.is_add_maker());
                let expected_order = DummyOrder::from_order(expected_mutation.as_add_maker());
                expected_order.verify(real_mutation.as_add_maker(), false);
            } else if expected_mutation.is_trade() {
                assert!(real_mutation.is_trade());
                let expected_trade = expected_mutation.as_trade();
                verify_trade(
                    real_mutation.as_trade(),
                    &DummyOrder::from_order(expected_trade.taker()),
                    &DummyOrder::from_order(expected_trade.maker()),
                    expected_trade.price(),
                    expected_trade.qty(),
                );
            }
        }
    }

    #[test]
    fn simple_add_order() {
        let mut ledger = Ledger::default();
        let dummy_order = DummyOrder::new(1234567890, 999, 111, SideT::BUY, OrderTypeT::LIMIT);
        verify_mutations(
            &ledger.add_order(dummy_order.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_order.to_order())),
        );
        let buy_ledger = ledger.buy_ledger_for_test();
        let sell_ledger = ledger.sell_ledger_for_test();
        assert_eq!(buy_ledger.len(), 1);
        assert_eq!(sell_ledger.len(), 0);
        if let Some(ref order) = buy_ledger.peek() {
            dummy_order.verify(order, false);
        } else {
            panic!("Expected valid item in ledger.");
        }
    }

    #[test]
    fn simple_full_trade() {
        let mut ledger = Ledger::default();
        let dummy_buy = DummyOrder::new(1234567890, 10, 8, SideT::BUY, OrderTypeT::LIMIT);
        let dummy_sell = DummyOrder::new(987654321, 10, 8, SideT::SELL, OrderTypeT::LIMIT);

        verify_mutations(
            &ledger.add_order(dummy_buy.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_sell.to_order()),
            &vec!(
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy.to_order())),
            ),
        );
        assert_eq!(ledger.buy_ledger_for_test().len(), 0);
        assert_eq!(ledger.sell_ledger_for_test().len(), 0);
    }

    #[test]
    fn market_order_fails_if_empty() {
        let mut ledger = Ledger::default();
        let dummy_buy = DummyOrder::new(1234567890, 10, 8, SideT::BUY, OrderTypeT::MARKET);
        let dummy_sell = DummyOrder::new(987654321, 10, 8, SideT::BUY, OrderTypeT::MARKET);
        {
            let error = ledger.add_order(dummy_buy.to_order()).err().unwrap();
            assert_eq!(error, AddOrderError::NotEnoughOrdersToFillMarketOrder);
        }
        {
            let error = ledger.add_order(dummy_sell.to_order()).err().unwrap();
            assert_eq!(error, AddOrderError::NotEnoughOrdersToFillMarketOrder);
        }
        assert_eq!(ledger.buy_ledger_for_test().len(), 0);
        assert_eq!(ledger.sell_ledger_for_test().len(), 0);
    }

    #[test]
    fn buy_limits_inserted_backwards_traded_in_order() {
        let mut ledger = Ledger::default();
        let dummy_buy1 = DummyOrder::new(99, 14, 1, SideT::BUY, OrderTypeT::LIMIT);
        let dummy_buy2 = DummyOrder::new(88, 13, 2, SideT::BUY, OrderTypeT::LIMIT);
        let dummy_buy3 = DummyOrder::new(77, 12, 3, SideT::BUY, OrderTypeT::LIMIT);
        let dummy_buy4 = DummyOrder::new(66, 11, 4, SideT::BUY, OrderTypeT::LIMIT);

        verify_mutations(
            &ledger.add_order(dummy_buy1.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy1.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_buy2.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy2.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_buy3.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy3.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_buy4.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy4.to_order())),
        );

        let dummy_sell = DummyOrder::new(5, 1, 10, SideT::SELL, OrderTypeT::LIMIT);
        verify_mutations(
            &ledger.add_order(dummy_sell.to_order()),
            &vec!(
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy1.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy2.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy3.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy4.to_order())),
            ),
        );
        assert_eq!(ledger.buy_ledger_for_test().len(), 0);
        assert_eq!(ledger.sell_ledger_for_test().len(), 0);
    }

    #[test]
    fn buy_limits_inserted_forwards_traded_in_order() {
        let mut ledger = Ledger::default();
        let dummy_buy1 = DummyOrder::new(99, 14, 1, SideT::BUY, OrderTypeT::LIMIT);
        let dummy_buy2 = DummyOrder::new(88, 13, 2, SideT::BUY, OrderTypeT::LIMIT);
        let dummy_buy3 = DummyOrder::new(77, 12, 3, SideT::BUY, OrderTypeT::LIMIT);
        let dummy_buy4 = DummyOrder::new(66, 11, 4, SideT::BUY, OrderTypeT::LIMIT);

        // Notice they are inverted.
        verify_mutations(
            &ledger.add_order(dummy_buy4.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy4.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_buy3.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy3.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_buy2.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy2.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_buy1.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy1.to_order())),
        );

        let dummy_sell = DummyOrder::new(5, 1, 10, SideT::SELL, OrderTypeT::LIMIT);
        verify_mutations(
            &ledger.add_order(dummy_sell.to_order()),
            &vec!(
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy1.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy2.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy3.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy4.to_order())),
            ),
        );
        assert_eq!(ledger.buy_ledger_for_test().len(), 0);
        assert_eq!(ledger.sell_ledger_for_test().len(), 0);
    }

    #[test]
    fn buy_limits_inserted_random_traded_in_order() {
        let mut ledger = Ledger::default();
        let dummy_buy1 = DummyOrder::new(99, 14, 1, SideT::BUY, OrderTypeT::LIMIT);
        let dummy_buy2 = DummyOrder::new(88, 13, 2, SideT::BUY, OrderTypeT::LIMIT);
        let dummy_buy3 = DummyOrder::new(77, 12, 3, SideT::BUY, OrderTypeT::LIMIT);
        let dummy_buy4 = DummyOrder::new(66, 11, 4, SideT::BUY, OrderTypeT::LIMIT);

        // Notice they are random.
        verify_mutations(
            &ledger.add_order(dummy_buy3.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy3.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_buy1.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy1.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_buy2.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy2.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_buy4.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_buy4.to_order())),
        );

        let dummy_sell = DummyOrder::new(5, 1, 10, SideT::SELL, OrderTypeT::LIMIT);
        verify_mutations(
            &ledger.add_order(dummy_sell.to_order()),
            &vec!(
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy1.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy2.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy3.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell.to_order(), dummy_buy4.to_order())),
            ),
        );
        assert_eq!(ledger.buy_ledger_for_test().len(), 0);
        assert_eq!(ledger.sell_ledger_for_test().len(), 0);
    }

    #[test]
    fn sell_limits_inserted_backwards_traded_in_order() {
        let mut ledger = Ledger::default();
        let dummy_sell1 = DummyOrder::new(99, 14, 1, SideT::SELL, OrderTypeT::LIMIT);
        let dummy_sell2 = DummyOrder::new(88, 13, 2, SideT::SELL, OrderTypeT::LIMIT);
        let dummy_sell3 = DummyOrder::new(77, 12, 3, SideT::SELL, OrderTypeT::LIMIT);
        let dummy_sell4 = DummyOrder::new(66, 11, 4, SideT::SELL, OrderTypeT::LIMIT);

        verify_mutations(
            &ledger.add_order(dummy_sell1.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell1.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_sell2.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell2.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_sell3.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell3.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_sell4.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell4.to_order())),
        );

        let dummy_buy = DummyOrder::new(5, 100, 10, SideT::BUY, OrderTypeT::LIMIT);
        verify_mutations(
            &ledger.add_order(dummy_buy.to_order()),
            &vec!(
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell4.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell3.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell2.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell1.to_order())),
            ),
        );
        assert_eq!(ledger.buy_ledger_for_test().len(), 0);
        assert_eq!(ledger.sell_ledger_for_test().len(), 0);
    }

    #[test]
    fn sell_limits_inserted_forwards_traded_in_order() {
        let mut ledger = Ledger::default();
        let dummy_sell1 = DummyOrder::new(99, 14, 1, SideT::SELL, OrderTypeT::LIMIT);
        let dummy_sell2 = DummyOrder::new(88, 13, 2, SideT::SELL, OrderTypeT::LIMIT);
        let dummy_sell3 = DummyOrder::new(77, 12, 3, SideT::SELL, OrderTypeT::LIMIT);
        let dummy_sell4 = DummyOrder::new(66, 11, 4, SideT::SELL, OrderTypeT::LIMIT);

        // Notice they are inverted.
        verify_mutations(
            &ledger.add_order(dummy_sell4.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell4.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_sell3.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell3.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_sell2.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell2.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_sell1.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell1.to_order())),
        );

        let dummy_buy = DummyOrder::new(5, 100, 10, SideT::BUY, OrderTypeT::LIMIT);
        verify_mutations(
            &ledger.add_order(dummy_buy.to_order()),
            &vec!(
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell4.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell3.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell2.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell1.to_order())),
            ),
        );
        assert_eq!(ledger.buy_ledger_for_test().len(), 0);
        assert_eq!(ledger.sell_ledger_for_test().len(), 0);
    }

    #[test]
    fn sell_limits_inserted_random_traded_in_order() {
        let mut ledger = Ledger::default();
        let dummy_sell1 = DummyOrder::new(99, 14, 1, SideT::SELL, OrderTypeT::LIMIT);
        let dummy_sell2 = DummyOrder::new(88, 13, 2, SideT::SELL, OrderTypeT::LIMIT);
        let dummy_sell3 = DummyOrder::new(77, 12, 3, SideT::SELL, OrderTypeT::LIMIT);
        let dummy_sell4 = DummyOrder::new(66, 11, 4, SideT::SELL, OrderTypeT::LIMIT);

        // Notice they are random.
        verify_mutations(
            &ledger.add_order(dummy_sell3.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell3.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_sell1.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell1.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_sell2.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell2.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_sell4.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell4.to_order())),
        );

        let dummy_buy = DummyOrder::new(5, 100, 10, SideT::BUY, OrderTypeT::LIMIT);
        verify_mutations(
            &ledger.add_order(dummy_buy.to_order()),
            &vec!(
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell4.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell3.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell2.to_order())),
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell1.to_order())),
            ),
        );
        assert_eq!(ledger.buy_ledger_for_test().len(), 0);
        assert_eq!(ledger.sell_ledger_for_test().len(), 0);
    }

    #[test]
    fn buy_limit_consumes_and_goes_onto_ledger_then_consumed_by_market() {
        let mut ledger = Ledger::default();
        let dummy_sell1 = DummyOrder::new(99, 14, 1, SideT::SELL, OrderTypeT::LIMIT);
        let dummy_sell2 = DummyOrder::new(88, 17, 1, SideT::SELL, OrderTypeT::LIMIT);

        verify_mutations(
            &ledger.add_order(dummy_sell1.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell1.to_order())),
        );
        verify_mutations(
            &ledger.add_order(dummy_sell2.to_order()),
            &vec!(LedgerMutation::AddedMakerOrder(dummy_sell2.to_order())),
        );

        let dummy_buy = DummyOrder::new(77, 16, 3, SideT::BUY, OrderTypeT::LIMIT);
        let result_buy = DummyOrder::new(77, 16, 2, SideT::BUY, OrderTypeT::LIMIT);
        verify_mutations(
            &ledger.add_order(dummy_buy.to_order()),
            &vec!(
                LedgerMutation::TradeExecuted(Trade::new(dummy_buy.to_order(), dummy_sell1.to_order())),
                LedgerMutation::AddedMakerOrder(result_buy.to_order()),
            ),
        );

        let dummy_sell3 = DummyOrder::new(66, 1, 2, SideT::SELL, OrderTypeT::MARKET);
        // let result_sell = DummyOrder::new(77, 16, 2, SideT::BUY, OrderTypeT::LIMIT);
        verify_mutations(
            &ledger.add_order(dummy_sell3.to_order()),
            &vec!(
                LedgerMutation::TradeExecuted(Trade::new(dummy_sell3.to_order(), dummy_buy.to_order())),
                // LedgerMutation::AddedMakerOrder(result_buy.to_order()),
            ),
        );
        assert_eq!(ledger.buy_ledger_for_test().len(), 0);
        assert_eq!(ledger.sell_ledger_for_test().len(), 1);
        dummy_sell2.verify(ledger.sell_ledger_for_test().peek().unwrap(), true);
    }
}
