use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

use crate::{aliases::*, parse_as, sol_type};
use alloy_primitives::U8;
use alloy_sol_macro::sol;
use alloy_sol_types::{sol_data::FixedArray, SolEnum};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use substreams::Hex;
use substreams_ethereum::pb::eth::v2::Block;

// helper macro to populate a solidity json struct
macro_rules! json_sol {
    ($kind: expr, $val: ident) => {
        SolidityJsonValue {
            kind: $kind.to_string(),
            value: ValueKind::Scalar($val.to_string()),
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

macro_rules! impl_to {
    ($input: ty, $variant: ident) => {
        impl From<SolidityType> for $input {
            fn from(val: SolidityType) -> Self {
                if let SolidityType::$variant(value) = val {
                    value.into()
                } else {
                    panic!("Couldn't convert into variant!");
                }
            }
        }
    };
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SolidityJsonValue {
    /// Represents the type of the value
    kind: String,
    /// A hex string for the bytes in the value
    #[serde(flatten)]
    value: ValueKind,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ValueKind {
    #[serde(rename = "value")]
    Scalar(String),
    #[serde(rename = "value")]
    Compound(Vec<SolidityJsonValue>),
    #[serde(rename = "value")]
    Map(HashMap<String, SolidityJsonValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum SolidityType {
    Boolean(U1),
    Enum(U8),
    Uint(U256),
    Address(Address),
    ByteArray(Bytes),
    FixedArray(alloy_primitives::B256),
    String(String),
    Tuple(Vec<SolidityType>),
    List(Vec<SolidityType>),
    Struct(HashMap<String, SolidityType>),
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
            SolidityType::Tuple(val) => {
                let val: Vec<SolidityJsonValue> =
                    val.into_iter().map(|item| item.to_json_value()).collect();
                SolidityJsonValue {
                    kind: "tuple".to_string(),
                    value: ValueKind::Compound(val),
                }
            }
            SolidityType::List(val) => {
                let val: Vec<SolidityJsonValue> =
                    val.into_iter().map(|item| item.to_json_value()).collect();
                SolidityJsonValue {
                    kind: "list".to_string(),
                    value: ValueKind::Compound(val),
                }
            }
            SolidityType::Struct(val) => {
                let val: HashMap<String, SolidityJsonValue> = val
                    .into_iter()
                    .map(|(k, v)| (k, v.to_json_value()))
                    .collect();

                SolidityJsonValue {
                    kind: "struct".to_string(),
                    value: ValueKind::Map(val),
                }
            }
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
            SolidityType::Tuple(_) => panic!("Can't convert a tuple to a string!"),
            SolidityType::List(_) => panic!("Can't convert a list to a string!"),
            SolidityType::Struct(_) => panic!("Can't convert a struct to a string!"),
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
impl From<Value> for SolidityType {
    fn from(value: Value) -> Self {
        let as_json = SolidityJsonValue::from_value(&value).expect(&format!(
            "Couldn't convert value {value:?} into SolidityJsonValue!"
        ));
        as_json.into()
    }
}

// NOTE I might want to change this to try_from
impl From<Option<Value>> for SolidityType {
    fn from(value: Option<Value>) -> Self {
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
            "list" => {
                let value = self.value;
                if let ValueKind::Compound(vals) = value {
                    let vals = vals.into_iter().map(|item| item.to_sol_type()).collect();
                    SolidityType::List(vals)
                } else {
                    panic!("Invalid cast to a sol type");
                }
            }
            "tuple" => {
                let value = self.value;
                if let ValueKind::Compound(vals) = value {
                    let vals: Vec<SolidityType> =
                        vals.into_iter().map(|item| item.to_sol_type()).collect();
                    SolidityType::Tuple(vals)
                } else {
                    panic!("Invalid cast to a sol type");
                }
            }
            "struct" => {
                let value = self.value;

                if let ValueKind::Map(map) = value {
                    let map: HashMap<String, SolidityType> =
                        map.into_iter().map(|(k, v)| (k, v.to_sol_type())).collect();
                    return SolidityType::Struct(map);
                }

                panic!("The value of a struct should never be a Scalar value!");
            }
            _ => panic!("Invalid cast to a sol type"),
        }
    }

    pub fn from_value(value: &Value) -> Option<SolidityJsonValue> {
        serde_json::from_value(value.clone()).ok()
    }

    /// This function takes in a serde json value, and tries to guess the solidity type it represents, if any.
    /// Note that this can't tell the difference between bytes values and uints because they are represented as hex values all the same.
    pub fn guess_json_value(value: &Value) -> Option<SolidityJsonValue> {
        match value {
            Value::Bool(val) => Some(val.clone().into()),
            Value::String(val) => {
                // Address Check
                if val.starts_with("0x") && val.len() == 42 {
                    return Some(sol_type!(Address, val));
                }

                // Bytes32 / Uint256 Check
                // NOTE We are going to treat these as uints for now. Might want to change to bytes?
                if val.starts_with("0x") && val.len() == 66 {
                    return Some(sol_type!(Uint, val));
                }

                // Bytes check
                if val.starts_with("0x") && val.len() > 66 {
                    return Some(sol_type!(ByteArray, val));
                }

                // If the value starts with 0x, but all the other values failed
                if val.starts_with("0x") {
                    Some(sol_type!(Uint, val))
                } else {
                    // Otherwise we treat it as a String
                    Some(sol_type!(String, val))
                }
            }

            Value::Object(val) => {
                // tuple check
                let mut keys = val.keys();
                let key_regex = Regex::new(r"_\d+").unwrap();
                let keys_match = keys.all(|key| {
                    let re_match = key_regex.find(key);
                    if let Some(re_match) = re_match {
                        re_match.as_str() == key
                    } else {
                        false
                    }
                });

                // if the keys match the pattern of _0, _1, etc, it's a tuple.
                if keys_match {
                    let values: Vec<SolidityType> = val
                        .values()
                        .map(|value| SolidityJsonValue::guess_json_value(value).unwrap().into()) // TODO Slow, but fine for now
                        .collect();
                    if values.len() == 1 {
                        return Some(values[0].clone().into());
                    } else {
                        return Some(SolidityType::Tuple(values).into());
                    }
                } else {
                    // Otherwise if they don't match, it's a struct
                    let kvs = val
                        .into_iter()
                        .map(|(key, value)| {
                            (
                                key.to_string(),
                                SolidityJsonValue::guess_json_value(value).unwrap().into(),
                            )
                        })
                        .collect::<HashMap<String, SolidityType>>();
                    return Some(SolidityType::Struct(kvs).to_json_value());
                }
            }
            Value::Array(arr) => {
                // TODO Slow, but fine for now
                let values: Vec<SolidityType> = arr
                    .into_iter()
                    .map(|value| SolidityJsonValue::guess_json_value(value).unwrap().into())
                    .collect();

                Some(SolidityType::List(values).into())
            }
            Value::Null => todo!("Null types shouldn't be returned?"),
            // NOTE The only time this is returned is a number less than i32
            Value::Number(num) => {
                Some(SolidityType::Uint(U256::from(num.as_i64().unwrap())).into())
            }
        }
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
impl_to!(Address, Address);

impl_from!(String, String);
impl_to!(String, String);

impl_from!(U1, Boolean);

impl_from!(alloy_primitives::U256, Uint);
impl_to!(alloy_primitives::U256, Uint);

impl_from!(Bytes, ByteArray);
impl_to!(Bytes, ByteArray);

impl_from!(alloy_primitives::B256, FixedArray);
impl_to!(alloy_primitives::B256, FixedArray);

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

impl From<SolidityType> for bool {
    fn from(value: SolidityType) -> Self {
        if let SolidityType::Boolean(val) = value {
            let value: u8 = val.to();
            if value == 0 {
                false
            } else {
                true
            }
        } else {
            panic!("Tried to convert a non boolean value into a boolean!");
        }
    }
}

impl From<bool> for SolidityJsonValue {
    fn from(value: bool) -> Self {
        let value: SolidityType = value.into();
        value.into()
    }
}

impl From<SolidityJsonValue> for bool {
    fn from(value: SolidityJsonValue) -> Self {
        value.to_sol_type().into()
    }
}

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
            (SolidityType::String(lh), SolidityType::String(rh)) => lh.partial_cmp(rh),
            (SolidityType::Address(lh), SolidityType::Address(rh)) => lh.partial_cmp(rh),
            _ => panic!("Can't compare {self:?} and {other:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::address;

    use super::*;

    #[test]
    fn test_serializations() {
        let solidity_value = SolidityType::from(false);
        let as_value = serde_json::to_string_pretty(&solidity_value).unwrap();
        println!("Bool: {}", &as_value);
        let from_value: SolidityType = serde_json::from_str(&&as_value).unwrap();
        println!("Bool Deserialized: {:?}", &from_value);

        let solidity_value = SolidityType::Enum(U8::from(2));
        let as_value = serde_json::to_string_pretty(&solidity_value).unwrap();
        println!("Enum: {}", &as_value);
        let from_value: SolidityType = serde_json::from_str(&&as_value).unwrap();
        println!("Enum Deserialized: {:?}", &from_value);

        let solidity_value = SolidityType::Uint(U256::from(42069));
        let as_value = serde_json::to_string_pretty(&solidity_value).unwrap();
        println!("Uint: {}", &as_value);
        let from_value: SolidityType = serde_json::from_str(&&as_value).unwrap();
        println!("Uint Deserialized: {:?}", &from_value);

        let solidity_value =
            SolidityType::Address(address!("000000000000Ad05Ccc4F10045630fb830B95127"));
        let as_value = serde_json::to_string_pretty(&solidity_value).unwrap();
        println!("Address: {}", &as_value);
        let from_value: SolidityType = serde_json::from_str(&&as_value).unwrap();
        println!("Address Deserialized: {:?}", &from_value);

        // TODO BYTES!
        // TODO FIXEDARRAY!

        let solidity_value = SolidityType::from("Hello World!".to_string());
        let as_value = serde_json::to_string_pretty(&solidity_value).unwrap();
        println!("String: {}", &as_value);
        let from_value: SolidityType = serde_json::from_str(&&as_value).unwrap();
        println!("String Deserialized: {:?}", &from_value);

        let solidity_value = SolidityType::Tuple(vec![
            SolidityType::from(false),
            SolidityType::Address(address!("000000000000Ad05Ccc4F10045630fb830B95127")),
        ]);
        let as_value = serde_json::to_string_pretty(&solidity_value).unwrap();
        println!("Tuple: {}", &as_value);
        let from_value: SolidityType = serde_json::from_str(&&as_value).unwrap();
        println!("Tuple Deserialized: {:?}", &from_value);

        let solidity_value = SolidityType::List(vec![
            SolidityType::from(false),
            SolidityType::Address(address!("000000000000Ad05Ccc4F10045630fb830B95127")),
        ]);
        let as_value = serde_json::to_string_pretty(&solidity_value).unwrap();
        println!("List: {}", &as_value);
        let from_value: SolidityType = serde_json::from_str(&&as_value).unwrap();
        println!("List Deserialized: {:?}", &from_value);

        let mut struct_map: HashMap<String, SolidityType> = HashMap::new();
        struct_map.insert("bool".to_string(), SolidityType::from(false));
        struct_map.insert(
            "addr".to_string(),
            SolidityType::Address(address!("000000000000Ad05Ccc4F10045630fb830B95127")),
        );
        struct_map.insert(
            "foo".to_string(),
            SolidityType::List(vec![
                SolidityType::from(false),
                SolidityType::Address(address!("000000000000Ad05Ccc4F10045630fb830B95127")),
            ]),
        );

        let solidity_value = SolidityType::Struct(struct_map);
        let as_value = serde_json::to_string_pretty(&solidity_value).unwrap();
        println!("Map: {}", &as_value);
        let from_value: SolidityType = serde_json::from_str(&&as_value).unwrap();
        println!("Map Deserialized: {:?}", &from_value);
    }
}
