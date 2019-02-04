use std::borrow::Cow;
use std::sync::mpsc::{Receiver, Sender};

use quick_protobuf::BytesReader;

use proto;

use super::*;

extern crate uuid;

pub fn start(tx: Sender<MatcherThreadActionMessage>, rx: Receiver<IoThreadMessage>) {
    IoThread::new(tx).run(rx);
    println!("IO thread exited.");
}

impl IoThread {
    pub fn new(tx: Sender<MatcherThreadActionMessage>) -> IoThread {
        IoThread { tx_to_matcher: tx }
    }

    pub fn run(&mut self, rx: Receiver<IoThreadMessage>) {
        for task_data in rx.iter() {
            match task_data {
                IoThreadMessage::ProcessRawData(data) => self.process_raw_data(&data),
                IoThreadMessage::MatcherActionResult(response) => {
                    self.process_action_result(response)
                }
                IoThreadMessage::Shutdown => break,
            }
        }
    }

    fn process_raw_data(&mut self, data: &[u8]) {
        let mut reader = BytesReader::from_bytes(&data);

        let any_proto = match reader.read_message::<proto::Any>(&data) {
            Ok(any_proto) => any_proto,
            Err(e) => {
                println!("Received bad/corrupted data: {}", e);
                return;
            }
        };
        let type_url = match any_proto.type_url {
            Some(type_url) => type_url,
            None => {
                println!("Received bad/corrupted data");
                return;
            }
        };
        if type_url != "libtradeengine.proto.Actions" {
            // TODO(nathan.bruer) Send this error back to sender thread via proto.
            println!("Received unsupported type '{}'", type_url);
            return;
        }
        let proto_value = match any_proto.value {
            Some(proto_value) => proto_value,
            None => {
                println!("Received bad/corrupted data");
                return;
            }
        };

        let mut actions_reader = BytesReader::from_bytes(&proto_value);
        let actions_proto = match actions_reader.read_message::<proto::Actions>(&proto_value) {
            Ok(action_proto) => action_proto,
            Err(e) => {
                println!("Received bad/corrupted data: {}", e);
                return;
            }
        };

        use proto::mod_Actions::OneOfaction_oneof;
        match actions_proto.action_oneof {
            OneOfaction_oneof::add_order(add_order_data) => self.process_add_order(&add_order_data),
            _ => {
                // TODO(nathan.bruer) Send this error back to sender thread via proto.
                println!("Unknown action");
                return;
            }
        }
        .unwrap();
    }

    fn sanitize_order<'a, 'b>(order: &'a proto::Order) -> Result<engine::Order, &'b str> {
        let customer_tag = order.customer_tag.as_ref().unwrap_or(&Cow::Borrowed(""));
        if customer_tag.len() > 32 {
            return Err("'customer_tag' must be <= 32 characters");
        }
        let price = order.price.unwrap_or(0);
        if price == 0 {
            return Err("'price' is invalid");
        }
        let qty = order.qty.unwrap_or(0);
        if qty == 0 {
            return Err("'qty' is invalid");
        }
        let side = order.side.unwrap_or(proto::SideT::INVALID);
        if side == proto::SideT::INVALID {
            return Err("'side' is invalid");
        }
        let order_type = order.order_type.unwrap_or(proto::OrderTypeT::INVALID);
        if order_type == proto::OrderTypeT::INVALID {
            return Err("'order_type' is invalid");
        }
        let id = *uuid::Uuid::new_v4().as_bytes();
        return Ok(engine::Order::new(id, price, qty, side, order_type));
    }

    fn process_add_order(
        &mut self,
        add_order_data: &proto::AddOrder,
    ) -> Result<engine::OrderIdT, &str> {
        let proto_order = match &add_order_data.order {
            Some(proto_order) => proto_order,
            None => {
                return Err("order not found in proto.");
            }
        };

        let _currency_pair = match &add_order_data.currency_pair {
            Some(currency_pair) => currency_pair,
            None => {
                return Err("currency_pair not found in proto.");
            }
        };

        let order = match IoThread::sanitize_order(&proto_order) {
            Ok(order) => order,
            Err(e) => {
                return Err(e);
            }
        };

        let action_id = *uuid::Uuid::new_v4().as_bytes();
        self.tx_to_matcher
            .send(MatcherThreadActionMessage {
                action_id,
                action: engine::MatcherAction::AddOrder(order),
            })
            .unwrap();
        return Ok([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    }

    fn process_action_result(&mut self, _response: MatcherActionResponse) {}
}

struct IoThread {
    tx_to_matcher: Sender<MatcherThreadActionMessage>,
}
