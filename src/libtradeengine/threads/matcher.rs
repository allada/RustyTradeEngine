use std::string::String;
use std::vec::Vec;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use super::*;
use engine::AddOrderError;
use engine::Ledger;
use engine::LedgerMutation;
use engine::MatcherAction;
use engine::MatcherActionResult;
use engine::Order;

pub fn start(
    currency_pair: String,
    tx: Sender<IoThreadMessage>,
    rx: Receiver<MatcherThreadActionMessage>,
) {
    MatcherThread::new(currency_pair, tx).run(rx);
    println!("Matcher thread exited.");
}

impl MatcherThread {
    pub fn new(currency_pair: String, tx: Sender<IoThreadMessage>) -> MatcherThread {
        MatcherThread {
            currency_pair,
            tx_to_io: tx,
            ledger: Ledger::default(),
        }
    }

    pub fn run(&mut self, rx: Receiver<MatcherThreadActionMessage>) {
        let mut orders_received = 0;
        for action in rx.iter() {
            match action.action {
                MatcherAction::AddOrder(order) => {
                    // let order_id = *order.id();
                    // let order_copy = order.clone();
                    let add_order_result = self.handle_add_order(order);
                    self.tx_to_io
                        .send(IoThreadMessage::MatcherActionResult(
                            MatcherActionResponse {
                                action_id: action.action_id,
                                result: MatcherActionResult::AddOrder(add_order_result),
                            },
                        ))
                        .err();
                    // self.tx_to_io
                    //     .send(IoThreadMessage::TradesExecuted(trades))
                    //     .err();
                    orders_received += 1;
                    if orders_received % 10000 == 0 {
                        println!("{}: At: {}", self.currency_pair, orders_received);
                    }
                }
                MatcherAction::Shutdown => break,
            }
        }
        println!(
            "{}: Orders Received {}",
            self.currency_pair, orders_received
        );
    }

    fn handle_add_order(&mut self, order: Order) -> Result<Vec<LedgerMutation>, AddOrderError> {
        self.ledger.add_order(order)
    }
}

struct MatcherThread {
    // Name for debugging and tracking reasons on which matcher
    // thread this is.
    currency_pair: String,
    tx_to_io: Sender<IoThreadMessage>,
    ledger: Ledger,
}
