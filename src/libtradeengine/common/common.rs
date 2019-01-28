
use std::io::Write;
use quick_protobuf::{MessageRead, MessageWrite, BytesReader, Writer, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SideT {
    BUY = 1,
    SELL = 2,
}

impl Default for SideT {
    fn default() -> Self {
        SideT::BUY
    }
}

impl From<i32> for SideT {
    fn from(i: i32) -> Self {
        match i {
            1 => SideT::BUY,
            2 => SideT::SELL,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for SideT {
    fn from(s: &'a str) -> Self {
        match s {
            "BUY" => SideT::BUY,
            "SELL" => SideT::SELL,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OrderTypeT {
    MARKET = 1,
    LIMIT = 2,
    LIMIT_MAKER = 3,
}

impl Default for OrderTypeT {
    fn default() -> Self {
        OrderTypeT::MARKET
    }
}

impl From<i32> for OrderTypeT {
    fn from(i: i32) -> Self {
        match i {
            1 => OrderTypeT::MARKET,
            2 => OrderTypeT::LIMIT,
            3 => OrderTypeT::LIMIT_MAKER,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for OrderTypeT {
    fn from(s: &'a str) -> Self {
        match s {
            "MARKET" => OrderTypeT::MARKET,
            "LIMIT" => OrderTypeT::LIMIT,
            "LIMIT_MAKER" => OrderTypeT::LIMIT_MAKER,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Order {
    pub id: Option<u64>,
    pub price: Option<u64>,
    pub qty: Option<u64>,
    pub side: Option<common::SideT>,
    pub order_type: Option<common::OrderTypeT>,
}

impl<'a> MessageRead<'a> for Order {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = Some(r.read_uint64(bytes)?),
                Ok(16) => msg.price = Some(r.read_uint64(bytes)?),
                Ok(24) => msg.qty = Some(r.read_uint64(bytes)?),
                Ok(32) => msg.side = Some(r.read_enum(bytes)?),
                Ok(40) => msg.order_type = Some(r.read_enum(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Order {
    fn get_size(&self) -> usize {
        0
        + self.id.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.price.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.qty.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.side.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.order_type.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.id { w.write_with_tag(8, |w| w.write_uint64(*s))?; }
        if let Some(ref s) = self.price { w.write_with_tag(16, |w| w.write_uint64(*s))?; }
        if let Some(ref s) = self.qty { w.write_with_tag(24, |w| w.write_uint64(*s))?; }
        if let Some(ref s) = self.side { w.write_with_tag(32, |w| w.write_enum(*s as i32))?; }
        if let Some(ref s) = self.order_type { w.write_with_tag(40, |w| w.write_enum(*s as i32))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Trade {
    pub taker: Option<common::Order>,
    pub maker: Option<common::Order>,
}

impl<'a> MessageRead<'a> for Trade {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.taker = Some(r.read_message::<common::Order>(bytes)?),
                Ok(18) => msg.maker = Some(r.read_message::<common::Order>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Trade {
    fn get_size(&self) -> usize {
        0
        + self.taker.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.maker.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.taker { w.write_with_tag(10, |w| w.write_message(s))?; }
        if let Some(ref s) = self.maker { w.write_with_tag(18, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct AddOrder {
    pub currency_pair: Option<common::Order>,
    pub order: Option<common::Order>,
}

impl<'a> MessageRead<'a> for AddOrder {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.currency_pair = Some(r.read_message::<common::Order>(bytes)?),
                Ok(18) => msg.order = Some(r.read_message::<common::Order>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for AddOrder {
    fn get_size(&self) -> usize {
        0
        + self.currency_pair.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
        + self.order.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.currency_pair { w.write_with_tag(10, |w| w.write_message(s))?; }
        if let Some(ref s) = self.order { w.write_with_tag(18, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Actions {
    pub action_oneof: common::mod_Actions::OneOfaction_oneof,
}

impl<'a> MessageRead<'a> for Actions {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.action_oneof = common::mod_Actions::OneOfaction_oneof::add_order(r.read_message::<common::AddOrder>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Actions {
    fn get_size(&self) -> usize {
        0
        + match self.action_oneof {
            common::mod_Actions::OneOfaction_oneof::add_order(ref m) => 1 + sizeof_len((m).get_size()),
            common::mod_Actions::OneOfaction_oneof::None => 0,
    }    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        match self.action_oneof {            common::mod_Actions::OneOfaction_oneof::add_order(ref m) => { w.write_with_tag(10, |w| w.write_message(m))? },
            common::mod_Actions::OneOfaction_oneof::None => {},
    }        Ok(())
    }
}

pub mod mod_Actions {

use super::*;

#[derive(Debug, PartialEq, Clone)]
pub enum OneOfaction_oneof {
    add_order(common::AddOrder),
    None,
}

impl Default for OneOfaction_oneof {
    fn default() -> Self {
        OneOfaction_oneof::None
    }
}

}

