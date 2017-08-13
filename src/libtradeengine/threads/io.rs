use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use engine::Order;

use super::*;

pub fn start(tx: Sender<UiThreadMessage>, rx: Receiver<IoThreadMessage>) {
  IoThread::new(tx).run(rx);
  println!("IO thread exited.");
}

impl IoThread {
  pub fn new(tx: Sender<UiThreadMessage>) -> IoThread {
    IoThread{
      tx_to_ui: tx,
    }
  }

  pub fn run(&mut self, rx: Receiver<IoThreadMessage>) {
    let mut it = rx.iter();
    while let Some(task_data) = it.next() {
      match task_data {
        IoThreadMessage::AddOrderAck(id) => {},
        IoThreadMessage::Exit => break,
      }
    }
  }
}

struct IoThread {
  tx_to_ui: Sender<UiThreadMessage>,
}