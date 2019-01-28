use super::types::*;
use super::Ledger;
use super::Order;
use super::Trade;

impl DummyOrder {
    pub fn new(
        id: OrderIdT,
        price: PriceT,
        qty: QtyT,
        side: SideT,
        order_type: OrderTypeT,
    ) -> DummyOrder {
        DummyOrder {
            id: id,
            price: price,
            qty: qty,
            side: side,
            order_type: order_type,
        }
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
        trade_price: PriceT,
        trade_qty: QtyT,
    ) {
        let trade_taker = trade.taker();
        let trade_maker = trade.maker();
        taker.verify(trade_taker, true);
        maker.verify(trade_maker, true);
        assert_eq!(*trade.qty(), trade_qty, "Trade qty does not match.");
        assert_eq!(
            *trade.price(),
            trade_price,
            "Trade price does not match the input param."
        );
        assert_eq!(
            *maker.price(),
            trade_price,
            "Input param price is expected to be maker price."
        );
        assert!(
            *taker.qty() == trade_qty || *maker.qty() == trade_qty,
            "Expected one of the orders to be of trade_qty {}",
            trade_qty
        );
    }

    #[test]
    fn simple_add_order() {
        let mut ledger = Ledger::default();
        let dummy_order = DummyOrder::new(1234567890, 999, 111, SideT::BUY, OrderTypeT::LIMIT);
        {
            let trades = ledger.add_order(dummy_order.to_order());
            assert_eq!(trades.len(), 0);
        }
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
        {
            let trades = ledger.add_order(dummy_buy.to_order());
            assert_eq!(trades.len(), 0);
        }
        {
            let trades = ledger.add_order(dummy_sell.to_order());
            assert_eq!(trades.len(), 1);
            verify_trade(trades.last().unwrap(), &dummy_sell, &dummy_buy, 10, 8);
        }
        assert_eq!(ledger.buy_ledger_for_test().len(), 0);
        assert_eq!(ledger.sell_ledger_for_test().len(), 0);
    }

    #[test]
    fn market_order_fails_if_empty() {
        let mut ledger = Ledger::default();
        let dummy_buy = DummyOrder::new(1234567890, 10, 8, SideT::BUY, OrderTypeT::MARKET);
        let dummy_sell = DummyOrder::new(987654321, 10, 8, SideT::BUY, OrderTypeT::MARKET);
        {
            let trades = ledger.add_order(dummy_buy.to_order());
            assert_eq!(trades.len(), 0);
        }
        {
            let trades = ledger.add_order(dummy_sell.to_order());
            assert_eq!(trades.len(), 0);
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

        assert_eq!(ledger.add_order(dummy_buy1.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_buy2.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_buy3.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_buy4.to_order()).len(), 0);

        let dummy_sell = DummyOrder::new(5, 1, 10, SideT::SELL, OrderTypeT::LIMIT);
        {
            let trades = ledger.add_order(dummy_sell.to_order());
            assert_eq!(trades.len(), 4);
            verify_trade(&trades[0], &dummy_sell, &dummy_buy1, 14, 1);
            verify_trade(&trades[1], &dummy_sell, &dummy_buy2, 13, 2);
            verify_trade(&trades[2], &dummy_sell, &dummy_buy3, 12, 3);
            verify_trade(&trades[3], &dummy_sell, &dummy_buy4, 11, 4);
        }
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
        assert_eq!(ledger.add_order(dummy_buy4.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_buy3.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_buy2.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_buy1.to_order()).len(), 0);

        let dummy_sell = DummyOrder::new(5, 1, 10, SideT::SELL, OrderTypeT::LIMIT);
        {
            let trades = ledger.add_order(dummy_sell.to_order());
            assert_eq!(trades.len(), 4);
            verify_trade(&trades[0], &dummy_sell, &dummy_buy1, 14, 1);
            verify_trade(&trades[1], &dummy_sell, &dummy_buy2, 13, 2);
            verify_trade(&trades[2], &dummy_sell, &dummy_buy3, 12, 3);
            verify_trade(&trades[3], &dummy_sell, &dummy_buy4, 11, 4);
        }
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
        assert_eq!(ledger.add_order(dummy_buy3.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_buy1.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_buy2.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_buy4.to_order()).len(), 0);

        let dummy_sell = DummyOrder::new(5, 1, 10, SideT::SELL, OrderTypeT::LIMIT);
        {
            let trades = ledger.add_order(dummy_sell.to_order());
            assert_eq!(trades.len(), 4);
            verify_trade(&trades[0], &dummy_sell, &dummy_buy1, 14, 1);
            verify_trade(&trades[1], &dummy_sell, &dummy_buy2, 13, 2);
            verify_trade(&trades[2], &dummy_sell, &dummy_buy3, 12, 3);
            verify_trade(&trades[3], &dummy_sell, &dummy_buy4, 11, 4);
        }
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

        assert_eq!(ledger.add_order(dummy_sell1.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_sell2.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_sell3.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_sell4.to_order()).len(), 0);

        let dummy_buy = DummyOrder::new(5, 100, 10, SideT::BUY, OrderTypeT::LIMIT);
        {
            let trades = ledger.add_order(dummy_buy.to_order());
            assert_eq!(trades.len(), 4);
            verify_trade(&trades[0], &dummy_buy, &dummy_sell4, 11, 4);
            verify_trade(&trades[1], &dummy_buy, &dummy_sell3, 12, 3);
            verify_trade(&trades[2], &dummy_buy, &dummy_sell2, 13, 2);
            verify_trade(&trades[3], &dummy_buy, &dummy_sell1, 14, 1);
        }
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
        assert_eq!(ledger.add_order(dummy_sell4.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_sell3.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_sell2.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_sell1.to_order()).len(), 0);

        let dummy_buy = DummyOrder::new(5, 100, 10, SideT::BUY, OrderTypeT::LIMIT);
        {
            let trades = ledger.add_order(dummy_buy.to_order());
            assert_eq!(trades.len(), 4);
            verify_trade(&trades[0], &dummy_buy, &dummy_sell4, 11, 4);
            verify_trade(&trades[1], &dummy_buy, &dummy_sell3, 12, 3);
            verify_trade(&trades[2], &dummy_buy, &dummy_sell2, 13, 2);
            verify_trade(&trades[3], &dummy_buy, &dummy_sell1, 14, 1);
        }
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
        assert_eq!(ledger.add_order(dummy_sell3.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_sell1.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_sell2.to_order()).len(), 0);
        assert_eq!(ledger.add_order(dummy_sell4.to_order()).len(), 0);

        let dummy_buy = DummyOrder::new(5, 100, 10, SideT::BUY, OrderTypeT::LIMIT);
        {
            let trades = ledger.add_order(dummy_buy.to_order());
            assert_eq!(trades.len(), 4);
            verify_trade(&trades[0], &dummy_buy, &dummy_sell4, 11, 4);
            verify_trade(&trades[1], &dummy_buy, &dummy_sell3, 12, 3);
            verify_trade(&trades[2], &dummy_buy, &dummy_sell2, 13, 2);
            verify_trade(&trades[3], &dummy_buy, &dummy_sell1, 14, 1);
        }
        assert_eq!(ledger.buy_ledger_for_test().len(), 0);
        assert_eq!(ledger.sell_ledger_for_test().len(), 0);
    }

    #[test]
    fn buy_limit_consumes_and_goes_onto_ledger_then_consumed_by_market() {
        let mut ledger = Ledger::default();
        let dummy_sell1 = DummyOrder::new(99, 14, 1, SideT::SELL, OrderTypeT::LIMIT);
        let dummy_sell2 = DummyOrder::new(88, 17, 1, SideT::SELL, OrderTypeT::LIMIT);

        let dummy_buy = DummyOrder::new(77, 16, 3, SideT::BUY, OrderTypeT::LIMIT);
        {
            assert_eq!(ledger.add_order(dummy_sell1.to_order()).len(), 0);
            assert_eq!(ledger.add_order(dummy_sell2.to_order()).len(), 0);
            let trades = ledger.add_order(dummy_buy.to_order());
            assert_eq!(trades.len(), 1);
            verify_trade(&trades[0], &dummy_buy, &dummy_sell1, 14, 1);
        }
        {
            let dummy_sell3 = DummyOrder::new(66, 1, 2, SideT::SELL, OrderTypeT::MARKET);
            let trades = ledger.add_order(dummy_sell3.to_order());
            assert_eq!(trades.len(), 1);
            verify_trade(&trades[0], &dummy_sell3, &dummy_buy, 16, 2);
        }
        assert_eq!(ledger.buy_ledger_for_test().len(), 0);
        assert_eq!(ledger.sell_ledger_for_test().len(), 1);
        dummy_sell2.verify(ledger.sell_ledger_for_test().peek().unwrap(), true);
    }
}
