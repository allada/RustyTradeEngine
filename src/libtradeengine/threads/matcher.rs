use std::string::String;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use super::*;
use engine::Ledger;
use engine::Order;

pub fn start(
    currency_pair: String,
    tx: Sender<IoThreadMessage>,
    rx: Receiver<MatcherThreadMessage>,
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

    pub fn run(&mut self, rx: Receiver<MatcherThreadMessage>) {
        let mut orders_received = 0;
        for task_data in rx.iter() {
            match task_data {
                MatcherThreadMessage::AddOrder(order) => {
                    let order_id = *order.id();
                    self.handle_add_order(order);
                    self.tx_to_io
                        .send(IoThreadMessage::AddOrderAck(order_id))
                        .err();
                    orders_received += 1;
                    if orders_received % 10000 == 0 {
                        println!("{}: At: {}", self.currency_pair, orders_received);
                    }
                }
                MatcherThreadMessage::Exit => break,
            }
        }
        println!(
            "{}: Orders Received {}",
            self.currency_pair, orders_received
        );
    }

    fn handle_add_order(&mut self, order: Order) {
        self.ledger.add_order(order);
    }
}

struct MatcherThread {
    // Name for debugging and tracking reasons on which matcher
    // thread this is.
    currency_pair: String,
    tx_to_io: Sender<IoThreadMessage>,
    ledger: Ledger,
}
