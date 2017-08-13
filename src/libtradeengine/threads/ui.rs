use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use engine::Order;
use engine::Ledger;
use super::*;

pub fn start(tx: Sender<IoThreadMessage>, rx: Receiver<UiThreadMessage>) {
  UiThread::new(tx).run(rx);
  println!("UI thread exited.");
}

impl UiThread {
  pub fn new(tx: Sender<IoThreadMessage>) -> UiThread {
    UiThread{
      tx_to_io: tx,
      ledger: Ledger::new(),
    }
  }

  pub fn run(&mut self, rx: Receiver<UiThreadMessage>) {
    let mut it = rx.iter();
    let mut orders_received = 0;
    while let Some(task_data) = it.next() {
      match task_data {
        UiThreadMessage::AddOrder(order) => {
          let order_id = *order.id();
          self.handle_add_order(order);
          self.tx_to_io.send(IoThreadMessage::AddOrderAck(order_id)).err();
          orders_received += 1;
          if orders_received % 10000 == 0 {
            println!("At: {}", orders_received);
          }
        },
        UiThreadMessage::Exit => break,
      }
    }
    println!("Orders Received {}", orders_received);
  }

  fn handle_add_order(&mut self, order: Order) {
    self.ledger.add_order(order);
  }
}

struct UiThread {
  tx_to_io: Sender<IoThreadMessage>,
  ledger: Ledger,
}