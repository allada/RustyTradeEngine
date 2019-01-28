use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use super::*;
use engine::Ledger;
use engine::Order;

pub fn start(tx: Sender<IoThreadMessage>, rx: Receiver<MatcherThreadMessage>) {
    MatcherThread::new(tx).run(rx);
    println!("Matcher thread exited.");
}

impl MatcherThread {
    pub fn new(tx: Sender<IoThreadMessage>) -> MatcherThread {
        MatcherThread {
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
                        println!("At: {}", orders_received);
                    }
                }
                MatcherThreadMessage::Exit => break,
            }
        }
        println!("Orders Received {}", orders_received);
    }

    fn handle_add_order(&mut self, order: Order) {
        self.ledger.add_order(order);
    }
}

struct MatcherThread {
    tx_to_io: Sender<IoThreadMessage>,
    ledger: Ledger,
}
