use crate::{aliases::*, parse_as};
use alloy_sol_macro::sol;
use alloy_sol_types::sol_data::FixedArray;
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

#[derive(Deserialize, Serialize)]
pub struct SolidityJsonValue {
    /// Represents the type of the value
    kind: String,
    /// A hex string for the bytes in the value
    value: String,
}

pub enum SolidityType {
    Boolean(U1),
    Uint(U256),
    Address(Address),
    ByteArray(Bytes),
    FixedArray(B32),
    String(String),
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
        }
    }
}

// NOTE I might want to change this to try_from
impl From<SolidityJsonValue> for SolidityType {
    fn from(value: SolidityJsonValue) -> Self {
        value.to_sol_type()
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
impl_from!(B32, FixedArray);

impl From<Vec<u8>> for SolidityType {
    fn from(value: Vec<u8>) -> Self {
        if value.len() == 32 {
            SolidityType::FixedArray(B32::from_slice(&value))
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
