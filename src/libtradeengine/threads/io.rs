use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use super::*;

pub fn start(tx: Sender<MatcherThreadMessage>, rx: Receiver<IoThreadMessage>) {
  IoThread::new(tx).run(rx);
  println!("IO thread exited.");
}

impl IoThread {
  pub fn new(tx: Sender<MatcherThreadMessage>) -> IoThread {
    IoThread{
      tx_to_matcher: tx,
    }
  }

  pub fn run(&mut self, rx: Receiver<IoThreadMessage>) {
    for task_data in rx.iter() {
      match task_data {
        IoThreadMessage::AddOrderAck(_id) => {},
        IoThreadMessage::Exit => break,
      }
    }
  }
}

struct IoThread {
  tx_to_matcher: Sender<MatcherThreadMessage>,
}