use std::borrow::Cow;
use std::sync::mpsc::channel;
use std::thread;

extern crate quick_protobuf;
extern crate tradeengine;

extern crate rand;
use self::rand::Rng;

use quick_protobuf::{serialize_into_vec};

use tradeengine::*;
// use threads;
// use threads::io;
// use threads::matcher;

fn main() {
    let (matcher_tx, matcher_rx) = channel::<threads::MatcherThreadMessage>();
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

    let mut rng = rand::thread_rng();
    for i in 0..1_000_000 {
        let side = if rng.gen() { proto::SideT::BUY } else { proto::SideT::SELL };
        let data = proto::Actions{
            id_guid: Some(Cow::Borrowed("blah")),
            action_oneof: proto::mod_Actions::OneOfaction_oneof::add_order(
                proto::AddOrder{
                    currency_pair: Some(Cow::Borrowed("debug_debug")),
                    order: Some(proto::Order{
                        id: Some(i + 1),
                        price: Some(i % 10 + 1),
                        qty: Some(i % 10 + 1),
                        side: Some(side),
                        order_type: Some(proto::OrderTypeT::LIMIT),
                    }),
                }),
        };
        let serialized_order = serialize_into_vec(&data).expect("Could not write order");
        let any_proto = proto::Any{
            type_url: Some(Cow::Borrowed("libtradeengine.proto.Actions")),
            value: Some(Cow::Borrowed(&serialized_order)),
        };
        thread_manager_io_tx.send(threads::IoThreadMessage::ProcessRawData(
            serialize_into_vec(&any_proto).expect("Could not write order"))).unwrap();

        // thread_manager_matcher_tx
        //     .send(threads::MatcherThreadMessage::AddOrder(Order::new(
        //         i,
        //         i % 10,
        //         i % 10,
        //         side,
        //         OrderTypeT::LIMIT,
        //     )))
        //     .unwrap();
    }
    println!("Done Sending");

    thread_manager_matcher_tx
        .send(threads::MatcherThreadMessage::Exit)
        .unwrap();
    thread_manager_io_tx
        .send(threads::IoThreadMessage::Exit)
        .unwrap();

    //   let thread_io = thread::Builder::new().name("IO".into()).spawn(move || {
    //     tx.send(engine::Order::new(1, 1, 3, engine::SideT::BUY, engine::OrderTypeT::LIMIT)).unwrap();
    //     tx.send(engine::Order::new(2, 2, 3, engine::SideT::BUY, engine::OrderTypeT::LIMIT)).unwrap();
    //     tx.send(engine::Order::new(3, 3, 3, engine::SideT::BUY, engine::OrderTypeT::LIMIT)).unwrap();
    //     tx.send(engine::Order::new(4, 5, 3, engine::SideT::SELL, engine::OrderTypeT::MARKET)).unwrap();
    //   }).unwrap();

    //   let thread_matcher = thread::Builder::new().name("Matcher".into()).spawn(move || {
    //     let mut ledger = engine::Ledger::new();
    //     let mut it = rx.iter();
    //     while let Some(order) = it.next() {
    //       let trades = ledger.add_order(order);
    //       println!("{:?}", trades);
    //     }
    //     println!("Chanel was hung up.");
    //   }).unwrap();
    thread_io.join().unwrap();
    thread_matcher.join().unwrap();
    println!("Done?");
}
