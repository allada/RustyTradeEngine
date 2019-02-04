use std::borrow::Cow;
use std::sync::mpsc::channel;
use std::thread;

extern crate ctrlc;
extern crate quick_protobuf;
extern crate rand;
extern crate tradeengine;
extern crate uuid;

use rand::Rng;

use quick_protobuf::serialize_into_vec;

use tradeengine::*;

fn main() {
    let (matcher_tx, matcher_rx) = channel::<threads::MatcherThreadActionMessage>();
    let (io_tx, io_rx) = channel::<threads::IoThreadMessage>();

    let thread_manager_matcher_tx = matcher_tx.clone();
    let thread_manager_io_tx = io_tx.clone();

    let thread_io = thread::Builder::new()
        .name("IO".into())
        .spawn(move || threads::io::start(matcher_tx, io_rx))
        .unwrap();
    let thread_matcher = thread::Builder::new()
        .name("Matcher".into())
        .spawn(move || threads::matcher::start("debug_debug".into(), io_tx, matcher_rx))
        .unwrap();

    let sig_handler_io_tx = thread_manager_io_tx.clone();
    ctrlc::set_handler(move || {
        println!("Received Ctrl-C, shutting down gracefully...");
        sig_handler_io_tx
            .send(threads::IoThreadMessage::Shutdown)
            .unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    let mut rng = rand::thread_rng();
    for i in 0..1_000_000 {
        let side = if rng.gen() {
            proto::SideT::BUY
        } else {
            proto::SideT::SELL
        };
        let id_uuid = uuid::Uuid::new_v4().as_bytes().to_vec();
        let data = proto::Actions {
            id_uuid: Some(Cow::Owned(id_uuid)),
            action_oneof: proto::mod_Actions::OneOfaction_oneof::add_order(proto::AddOrder {
                currency_pair: Some(Cow::Borrowed("debug_debug")),
                order: Some(proto::Order {
                    customer_tag: Some(Cow::Owned((i + 1).to_string())),
                    price: Some(i % 10 + 1),
                    qty: Some(i % 10 + 1),
                    side: Some(side),
                    order_type: Some(proto::OrderTypeT::LIMIT),
                }),
            }),
        };
        let serialized_order = serialize_into_vec(&data).expect("Could not write order");
        let any_proto = proto::Any {
            type_url: Some(Cow::Borrowed("libtradeengine.proto.Actions")),
            value: Some(Cow::Borrowed(&serialized_order)),
        };
        thread_manager_io_tx
            .send(threads::IoThreadMessage::ProcessRawData(
                serialize_into_vec(&any_proto).expect("Could not write order"),
            ))
            .unwrap();
    }
    println!("Done Sending");

    thread_manager_io_tx
        .send(threads::IoThreadMessage::Shutdown)
        .unwrap();

    // Wait for io thread.
    thread_io.join().unwrap();

    thread_manager_matcher_tx
        .send(threads::MatcherThreadActionMessage{
            action_id: *uuid::Uuid::new_v4().as_bytes(),
            action: engine::MatcherAction::Shutdown
        })
        .unwrap();
    thread_matcher.join().unwrap();
    println!("Done.");
}
