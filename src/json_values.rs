use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

use crate::{aliases::*, map_literal, sol_type};
use alloy_primitives::U8;
use alloy_sol_macro::sol;
use alloy_sol_types::{sol_data::FixedArray, SolEnum};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use substreams::{
    pb::substreams::store_delta::Operation,
    store::{DeltaProto, Deltas},
    Hex,
};
use substreams_ethereum::pb::eth::v2::Block;

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
    #[serde(skip)]
    Null,
}

pub trait IntoSolType {
    fn into_sol_type(self) -> SolidityType;
}

impl SolidityType {
    /// This function takes in a serde json value, and tries to guess the solidity type it represents, if any.
    /// Note that this can't tell the difference between bytes values and uints because they are represented as hex values all the same.
    pub fn guess_json_value(value: &Value) -> Option<SolidityType> {
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
                        .map(|value| SolidityType::guess_json_value(value).unwrap()) // TODO Slow, but fine for now
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
                                SolidityType::guess_json_value(value).unwrap(),
                            )
                        })
                        .collect::<HashMap<String, SolidityType>>();
                    return Some(SolidityType::Struct(kvs));
                }
            }
            Value::Array(arr) => {
                // TODO Slow, but fine for now
                let values: Vec<SolidityType> = arr
                    .into_iter()
                    .map(|value| SolidityType::guess_json_value(value).unwrap())
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

    pub fn insert(&mut self, key: &str, value: SolidityType) -> Option<()> {
        match self {
            SolidityType::Tuple(ref mut val) => {
                let key = key
                    .parse()
                    .expect("Couldn't parse key into number for tuple insert!");
                val.insert(key, value);
                Some(())
            }
            SolidityType::List(ref mut list) => {
                let key = key
                    .parse()
                    .expect("Couldn't parse key into number for list insert!");
                list.insert(key, value);
                Some(())
            }
            SolidityType::Struct(ref mut map) => {
                map.insert(key.to_string(), value);
                Some(())
            }
            SolidityType::Boolean(_)
            | SolidityType::Enum(_)
            | SolidityType::Uint(_)
            | SolidityType::Address(_)
            | SolidityType::ByteArray(_)
            | SolidityType::FixedArray(_)
            | SolidityType::String(_)
            | SolidityType::Null => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<SolidityType> {
        match &self {
            SolidityType::Tuple(val) => {
                let key: usize = key
                    .parse()
                    .expect("Couldn't parse key into number for tuple insert!");
                let value: &SolidityType = val
                    .get(key)
                    .expect("No value found for key in tuple access!");
                Some(value.clone())
            }
            SolidityType::List(list) => {
                let key: usize = key
                    .parse()
                    .expect("Couldn't parse key into number for list insert!");
                let value: &SolidityType = list
                    .get(key)
                    .expect("No value found for key in list access!");
                Some(value.clone())
            }
            SolidityType::Struct(map) => {
                let value: &SolidityType = map
                    .get(key)
                    .expect("No value found for key in struct access!");
                Some(value.clone())
            }
            SolidityType::Boolean(_)
            | SolidityType::Enum(_)
            | SolidityType::Uint(_)
            | SolidityType::Address(_)
            | SolidityType::ByteArray(_)
            | SolidityType::FixedArray(_)
            | SolidityType::String(_)
            | SolidityType::Null => None,
        }
    }

    pub fn map<F>(&self, callback: F) -> SolidityType
    where
        F: Fn(&SolidityType) -> SolidityType,
    {
        match self {
            SolidityType::Tuple(vals) => {
                let values: Vec<SolidityType> =
                    vals.iter().map(callback).map(|item| item.clone()).collect();
                SolidityType::List(values)
            }
            SolidityType::List(list) => {
                let values: Vec<SolidityType> =
                    list.iter().map(callback).map(|item| item.clone()).collect();
                SolidityType::List(values)
            }
            SolidityType::Struct(_) => panic!("Tried to map over a struct!"),
            _ => panic!("Tried to map over a scalar value!"),
        }
    }

    pub fn filter<F>(&self, callback: F) -> SolidityType
    where
        F: Fn(&SolidityType) -> SolidityType,
    {
        match self {
            SolidityType::Tuple(vals) => {
                let values: Vec<SolidityType> = vals
                    .iter()
                    .filter_map(|item| {
                        let callback_value = callback(item);
                        if let SolidityType::Boolean(val) = callback_value {
                            let value: u8 = val.to();
                            if value == 0 {
                                None
                            } else {
                                Some(item.clone())
                            }
                        } else {
                            panic!("Tried to filter over a tuple, but found a non boolean value!")
                        }
                    })
                    .collect();
                SolidityType::List(values)
            }
            SolidityType::List(list) => {
                let values: Vec<SolidityType> = list
                    .iter()
                    .filter_map(|item| {
                        let callback_value = callback(item);
                        if let SolidityType::Boolean(val) = callback_value {
                            let value: u8 = val.to();
                            if value == 0 {
                                None
                            } else {
                                Some(item.clone())
                            }
                        } else {
                            panic!("Tried to filter over a tuple, but found a non boolean value!")
                        }
                    })
                    .collect();
                SolidityType::List(values)
            }
            SolidityType::Struct(_) => panic!("Tried to filter over a struct!"),
            _ => panic!("Tried to filter over a scalar value!"),
        }
    }

    pub fn to_maybe_value(&self) -> Option<Value> {
        match self {
            SolidityType::Tuple(vals) => {
                if vals.len() == 0 {
                    return None;
                } else {
                    Some(serde_json::to_value(self).unwrap())
                }
            }
            SolidityType::List(vals) => {
                if vals.len() == 0 {
                    return None;
                } else {
                    Some(serde_json::to_value(self).unwrap())
                }
            }
            SolidityType::Struct(map) => {
                let iter = map
                    .into_iter()
                    .map(|(k, v)| (k, v.to_maybe_value()))
                    .filter(|(_k, v)| v.is_some())
                    .map(|(k, v)| (k, v.unwrap()))
                    .collect::<Vec<_>>();

                let recur_iter = HashMap::<&String, Value>::from_iter(iter);

                if recur_iter.len() == 0 {
                    return None;
                } else {
                    Some(serde_json::to_value(recur_iter).unwrap())
                }
            }
            _ => Some(serde_json::to_value(self).unwrap()),
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
            SolidityType::Null => "null".to_string(),
            SolidityType::Tuple(_) => panic!("Can't convert a tuple to a string!"),
            SolidityType::List(_) => panic!("Can't convert a list to a string!"),
            SolidityType::Struct(_) => panic!("Can't convert a struct to a string!"),
        }
    }
}

impl From<prost_wkt_types::Struct> for SolidityType {
    fn from(value: prost_wkt_types::Struct) -> Self {
        let value = serde_json::to_value(value).unwrap();
        serde_json::from_value(value).unwrap()
    }
}

impl From<Deltas<DeltaProto<prost_wkt_types::Struct>>> for SolidityType {
    fn from(value: Deltas<DeltaProto<prost_wkt_types::Struct>>) -> Self {
        let deltas = value.deltas;
        let deltas = deltas.into_iter().map(SolidityType::from).collect();
        SolidityType::List(deltas)
    }
}

impl From<DeltaProto<prost_wkt_types::Struct>> for SolidityType {
    fn from(value: DeltaProto<prost_wkt_types::Struct>) -> Self {
        let DeltaProto {
            operation,
            key,
            old_value,
            new_value,
            ..
        } = value;

        map_literal!(
            "operation"; SolidityType::from(operation),
            "key"; SolidityType::from(key),
            "old_value"; SolidityType::from(old_value),
            "new_value"; SolidityType::from(new_value)
        )
    }
}

impl From<Operation> for SolidityType {
    fn from(value: Operation) -> Self {
        match value {
            Operation::Unset => SolidityType::from("Unset".to_string()),
            Operation::Create => SolidityType::from("Create".to_string()),
            Operation::Update => SolidityType::from("Update".to_string()),
            Operation::Delete => SolidityType::from("Update".to_string()),
        }
    }
}

// NOTE I might want to change this to try_from
impl From<Value> for SolidityType {
    fn from(value: Value) -> Self {
        serde_json::from_value(value).unwrap()
    }
}

impl From<&SolidityType> for SolidityType {
    fn from(value: &SolidityType) -> Self {
        value.into()
    }
}

// NOTE I might want to change this to try_from
impl From<Option<Value>> for SolidityType {
    fn from(value: Option<Value>) -> Self {
        serde_json::from_value(value.expect("Tried to convert a None value into a SolidityType!"))
            .unwrap()
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

pub fn format_hex(input: &[u8]) -> String {
    format!("0x{}", Hex(input).to_string())
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
