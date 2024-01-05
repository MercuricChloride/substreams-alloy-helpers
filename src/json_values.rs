use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

use crate::{aliases::*, parse_as};
use alloy_primitives::U8;
use alloy_sol_macro::sol;
use alloy_sol_types::{sol_data::FixedArray, SolEnum};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use substreams::Hex;
use substreams_ethereum::pb::eth::v2::Block;

// helper macro to populate a solidity json struct
macro_rules! json_sol {
    ($kind: expr, $val: ident) => {
        SolidityJsonValue {
            kind: $kind.to_string(),
            value: $val.to_string(),
        }
    };
}

// A helper macro to impl From<T> for solidity types
macro_rules! impl_from {
    ($input: ty, $variant: ident) => {
        impl From<$input> for SolidityType {
            fn from(val: $input) -> Self {
                SolidityType::$variant(val)
            }
        }
    };
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SolidityJsonValue {
    /// Represents the type of the value
    kind: String,
    /// A hex string for the bytes in the value
    value: String,
}

#[derive(Debug)]
pub enum SolidityType {
    Boolean(U1),
    Enum(U8),
    Uint(U256),
    Address(Address),
    ByteArray(Bytes),
    FixedArray(alloy_primitives::B256),
    String(String),
}

pub trait IntoSolType {
    fn into_sol_type(self) -> SolidityType;
}

impl SolidityType {
    pub fn to_json_value(self) -> SolidityJsonValue {
        match self {
            SolidityType::Boolean(val) => json_sol!("boolean", val),
            SolidityType::Uint(val) => json_sol!("uint", val),
            SolidityType::Address(val) => json_sol!("address", val),
            SolidityType::ByteArray(val) => json_sol!("bytes", val),
            SolidityType::FixedArray(val) => json_sol!("array", val),
            SolidityType::String(val) => json_sol!("string", val),
            SolidityType::Enum(val) => json_sol!("enum", val),
        }
    }

    pub fn add<T: Into<SolidityType>>(self, value: T) -> SolidityType {
        let value: SolidityType = value.into();
        match (&self, &value) {
            (SolidityType::Uint(lh), SolidityType::Uint(rh)) => {
                let sum = lh + rh;
                SolidityType::Uint(sum)
            }
            _ => panic!("Can't add {self:?} and {value:?}! Both values must be a uint!"),
        }
    }
}

impl ToString for SolidityType {
    fn to_string(&self) -> String {
        match &self {
            SolidityType::Boolean(val) => {
                let value: u8 = val.to();
                if value == 0 {
                    "false".to_string()
                } else {
                    "true".to_string()
                }
            }
            SolidityType::Enum(val) => {
                let value: u8 = val.to();
                value.to_string()
            }
            SolidityType::Uint(val) => val.to_string(),
            SolidityType::Address(val) => val.to_string(),
            SolidityType::ByteArray(val) => val.to_string(),
            SolidityType::FixedArray(val) => val.to_string(),
            SolidityType::String(val) => val.to_string(),
        }
    }
}

// NOTE I might want to change this to try_from
impl From<SolidityJsonValue> for SolidityType {
    fn from(value: SolidityJsonValue) -> Self {
        value.to_sol_type()
    }
}

// NOTE I might want to change this to try_from
impl From<serde_json::Value> for SolidityType {
    fn from(value: serde_json::Value) -> Self {
        let as_json = SolidityJsonValue::from_value(&value).expect(&format!(
            "Couldn't convert value {value:?} into SolidityJsonValue!"
        ));
        as_json.into()
    }
}

// NOTE I might want to change this to try_from
impl From<Option<serde_json::Value>> for SolidityType {
    fn from(value: Option<serde_json::Value>) -> Self {
        let as_json = SolidityJsonValue::from_value(&value.as_ref().unwrap()).expect(&format!(
            "Couldn't convert value {value:?} into SolidityJsonValue!"
        ));
        as_json.into()
    }
}

impl From<SolidityType> for SolidityJsonValue {
    fn from(value: SolidityType) -> Self {
        value.to_json_value()
    }
}

impl SolidityJsonValue {
    pub fn to_sol_type(self) -> SolidityType {
        match self.kind.as_str() {
            "boolean" => parse_as!(self, Boolean),
            "uint" => parse_as!(self, Uint),
            "bytes" => parse_as!(self, ByteArray),
            "string" => parse_as!(self, String),
            "address" => parse_as!(self, Address),
            _ => panic!("Invalid cast to a sol type"),
        }
    }

    pub fn from_value(value: &serde_json::Value) -> Option<SolidityJsonValue> {
        serde_json::from_value(value.clone()).ok()
    }
}

pub fn to_json_map<T: Serialize>(input: &T, map: &mut serde_json::Map<String, Value>) {
    let s = serde_json::to_string(input).unwrap();
    let json_map: Map<String, Value> = serde_json::from_str(&s).unwrap();

    for (k, v) in json_map.into_iter() {
        map.insert(k, v.to_string().into());
    }
}

pub fn block_meta(block: &Block) -> (&Vec<u8>, u64, u64) {
    let hash = &block.hash;
    let number = block.number;
    let timestamp = block.timestamp_seconds();
    (hash, number, timestamp)
}

macro_rules! json_insert {
    ($json: ident, $key: expr, $val: expr) => {
        $json.insert($key.to_string(), $val.clone().into());
    };
}

pub fn format_hex(input: &[u8]) -> String {
    format!("0x{}", Hex(input).to_string())
}

pub fn format_value(value: &ethereum_abi::Value) -> Value {
    match value {
        ethereum_abi::Value::Uint(val, _) => val.to_string().into(),
        ethereum_abi::Value::Int(val, _) => val.to_string().into(),
        ethereum_abi::Value::Address(val) => format_hex(val.as_bytes()).into(),
        ethereum_abi::Value::Bool(val) => (*val).into(),
        ethereum_abi::Value::FixedBytes(val) => format_hex(val).into(),
        ethereum_abi::Value::FixedArray(val, _) => {
            Value::Array(val.into_iter().map(format_value).collect())
        }
        ethereum_abi::Value::String(val) => val.clone().into(),
        ethereum_abi::Value::Bytes(val) => format_hex(val).into(),
        ethereum_abi::Value::Array(val, _) => {
            Value::Array(val.into_iter().map(format_value).collect())
        }
        ethereum_abi::Value::Tuple(val) => {
            let mut json = Map::new();
            for (key, value) in val.into_iter() {
                json_insert!(json, key, format_value(value));
            }
            json.into()
        }
    }
}

impl_from!(Address, Address);
impl_from!(String, String);
impl_from!(U1, Boolean);
impl_from!(alloy_primitives::U256, Uint);
impl_from!(Bytes, ByteArray);
impl_from!(alloy_primitives::B256, FixedArray);

impl From<Vec<u8>> for SolidityType {
    fn from(value: Vec<u8>) -> Self {
        if value.len() == 32 {
            SolidityType::FixedArray(alloy_primitives::B256::from_slice(&value))
        } else {
            SolidityType::ByteArray(Bytes::copy_from_slice(&value))
        }
    }
}

impl From<bool> for SolidityType {
    fn from(value: bool) -> Self {
        SolidityType::Boolean(U1::from(value))
    }
}

impl From<bool> for SolidityJsonValue {
    fn from(value: bool) -> Self {
        let value: SolidityType = value.into();
        value.into()
    }
}

// impl<T: SolEnum> From<T> for SolidityType {
//     fn from(value: T) -> Self {
//         let as_u8: u8 = value
//             .try_into()
//             .expect("couldn't convert an enum value to a u8!");
//         SolidityType::Enum(as_u8)
//     }
// }

// Binary Op Trait Implementations
impl<T> Add<T> for SolidityType
where
    SolidityType: From<T>,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        let rhs: SolidityType = Into::into(rhs);
        // NOTE If we add something to a string, it will concat. I'm not sure if I will keep this or not.
        if let SolidityType::String(str) = &self {
            let mut return_string = str.clone();
            return_string.push_str(&rhs.to_string());
            return SolidityType::String(return_string);
        }

        match (&self, &rhs) {
            (SolidityType::Uint(lh), SolidityType::Uint(rh)) => {
                let sum = lh + rh;
                SolidityType::Uint(sum)
            }
            _ => panic!("Can't add {self:?} and {rhs:?}! Both values must be a uint!"),
        }
    }
}

impl<T> Sub<T> for SolidityType
where
    SolidityType: From<T>,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        let rhs: SolidityType = Into::into(rhs);
        match (&self, &rhs) {
            (SolidityType::Uint(lh), SolidityType::Uint(rh)) => {
                let sum = lh - rh;
                SolidityType::Uint(sum)
            }
            _ => panic!("Can't add {self:?} and {rhs:?}! Both values must be a uint!"),
        }
    }
}

impl<T> Mul<T> for SolidityType
where
    SolidityType: From<T>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let rhs: SolidityType = Into::into(rhs);
        match (&self, &rhs) {
            (SolidityType::Uint(lh), SolidityType::Uint(rh)) => {
                let sum = lh * rh;
                SolidityType::Uint(sum)
            }
            _ => panic!("Can't add {self:?} and {rhs:?}! Both values must be a uint!"),
        }
    }
}

impl<T> Div<T> for SolidityType
where
    SolidityType: From<T>,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let rhs: SolidityType = Into::into(rhs);
        match (&self, &rhs) {
            (SolidityType::Uint(lh), SolidityType::Uint(rh)) => {
                let sum = lh / rh;
                SolidityType::Uint(sum)
            }
            _ => panic!("Can't add {self:?} and {rhs:?}! Both values must be a uint!"),
        }
    }
}

impl<T> PartialEq<T> for SolidityType
where
    SolidityType: From<T>,
    T: Clone + Debug,
{
    fn eq(&self, other: &T) -> bool {
        // TODO This isn't the most performant, but I don't think it's the end of the world
        let rhs: SolidityType = Into::into(other.clone());
        match (&self, &rhs) {
            (SolidityType::Uint(lh), SolidityType::Uint(rh)) => lh == rh,
            (SolidityType::Address(lh), SolidityType::Address(rh)) => lh == rh,
            _ => panic!("Can't compare {self:?} and {other:?}"),
        }
    }

    fn ne(&self, other: &T) -> bool {
        // TODO This isn't the most performant, but I don't think it's the end of the world
        let rhs: SolidityType = Into::into(other.clone());
        match (&self, &rhs) {
            (SolidityType::Uint(lh), SolidityType::Uint(rh)) => lh != rh,
            (SolidityType::Address(lh), SolidityType::Address(rh)) => lh != rh,
            _ => panic!("Can't compare {self:?} and {other:?}"),
        }
    }
}

impl<T> PartialOrd<T> for SolidityType
where
    SolidityType: From<T>,
    T: Clone + Debug,
{
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        // TODO This isn't the most performant, but I don't think it's the end of the world
        let rhs: SolidityType = Into::into(other.clone());
        match (&self, &rhs) {
            (SolidityType::Uint(lh), SolidityType::Uint(rh)) => lh.partial_cmp(rh),
            (SolidityType::Address(lh), SolidityType::Address(rh)) => lh.partial_cmp(rh),
            _ => panic!("Can't compare {self:?} and {other:?}"),
        }
    }
}
