
use std::io::Write;
use std::borrow::Cow;
use quick_protobuf::{MessageRead, MessageWrite, BytesReader, Writer, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SideT {
    INVALID = 0,
    BUY = 1,
    SELL = 2,
}

impl Default for SideT {
    fn default() -> Self {
        SideT::INVALID
    }
}

impl From<i32> for SideT {
    fn from(i: i32) -> Self {
        match i {
            0 => SideT::INVALID,
            1 => SideT::BUY,
            2 => SideT::SELL,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for SideT {
    fn from(s: &'a str) -> Self {
        match s {
            "INVALID" => SideT::INVALID,
            "BUY" => SideT::BUY,
            "SELL" => SideT::SELL,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OrderTypeT {
    INVALID = 0,
    MARKET = 1,
    LIMIT = 2,
    LIMIT_MAKER = 3,
}

impl Default for OrderTypeT {
    fn default() -> Self {
        OrderTypeT::INVALID
    }
}

impl From<i32> for OrderTypeT {
    fn from(i: i32) -> Self {
        match i {
            0 => OrderTypeT::INVALID,
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
            "INVALID" => OrderTypeT::INVALID,
            "MARKET" => OrderTypeT::MARKET,
            "LIMIT" => OrderTypeT::LIMIT,
            "LIMIT_MAKER" => OrderTypeT::LIMIT_MAKER,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Order<'a> {
    pub customer_tag: Option<Cow<'a, str>>,
    pub price: Option<u64>,
    pub qty: Option<u64>,
    pub side: Option<proto::SideT>,
    pub order_type: Option<proto::OrderTypeT>,
}

impl<'a> MessageRead<'a> for Order<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.customer_tag = Some(r.read_string(bytes).map(Cow::Borrowed)?),
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

impl<'a> MessageWrite for Order<'a> {
    fn get_size(&self) -> usize {
        0
        + self.customer_tag.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.price.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.qty.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.side.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.order_type.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.customer_tag { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.price { w.write_with_tag(16, |w| w.write_uint64(*s))?; }
        if let Some(ref s) = self.qty { w.write_with_tag(24, |w| w.write_uint64(*s))?; }
        if let Some(ref s) = self.side { w.write_with_tag(32, |w| w.write_enum(*s as i32))?; }
        if let Some(ref s) = self.order_type { w.write_with_tag(40, |w| w.write_enum(*s as i32))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Trade<'a> {
    pub taker: Option<proto::Order<'a>>,
    pub maker: Option<proto::Order<'a>>,
}

impl<'a> MessageRead<'a> for Trade<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.taker = Some(r.read_message::<proto::Order>(bytes)?),
                Ok(18) => msg.maker = Some(r.read_message::<proto::Order>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Trade<'a> {
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
pub struct AddOrder<'a> {
    pub currency_pair: Option<Cow<'a, str>>,
    pub order: Option<proto::Order<'a>>,
}

impl<'a> MessageRead<'a> for AddOrder<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.currency_pair = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(18) => msg.order = Some(r.read_message::<proto::Order>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for AddOrder<'a> {
    fn get_size(&self) -> usize {
        0
        + self.currency_pair.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.order.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.currency_pair { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.order { w.write_with_tag(18, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Actions<'a> {
    pub id_uuid: Option<Cow<'a, [u8]>>,
    pub action_oneof: proto::mod_Actions::OneOfaction_oneof<'a>,
}

impl<'a> MessageRead<'a> for Actions<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.id_uuid = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(18) => msg.action_oneof = proto::mod_Actions::OneOfaction_oneof::add_order(r.read_message::<proto::AddOrder>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Actions<'a> {
    fn get_size(&self) -> usize {
        0
        + self.id_uuid.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + match self.action_oneof {
            proto::mod_Actions::OneOfaction_oneof::add_order(ref m) => 1 + sizeof_len((m).get_size()),
            proto::mod_Actions::OneOfaction_oneof::None => 0,
    }    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.id_uuid { w.write_with_tag(10, |w| w.write_bytes(&**s))?; }
        match self.action_oneof {            proto::mod_Actions::OneOfaction_oneof::add_order(ref m) => { w.write_with_tag(18, |w| w.write_message(m))? },
            proto::mod_Actions::OneOfaction_oneof::None => {},
    }        Ok(())
    }
}

pub mod mod_Actions {

use super::*;

#[derive(Debug, PartialEq, Clone)]
pub enum OneOfaction_oneof<'a> {
    add_order(proto::AddOrder<'a>),
    None,
}

impl<'a> Default for OneOfaction_oneof<'a> {
    fn default() -> Self {
        OneOfaction_oneof::None
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Any<'a> {
    pub type_url: Option<Cow<'a, str>>,
    pub value: Option<Cow<'a, [u8]>>,
}

impl<'a> MessageRead<'a> for Any<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.type_url = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(18) => msg.value = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Any<'a> {
    fn get_size(&self) -> usize {
        0
        + self.type_url.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.value.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.type_url { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.value { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

