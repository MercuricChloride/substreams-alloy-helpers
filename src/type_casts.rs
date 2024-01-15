use crate::prelude::*;
use alloy_primitives::*;
use std::str::FromStr;

const ZERO_ADDRESS: Address = address!();

macro_rules! as_sol {
    (String, $val: expr) => {{
        let string = String::from($val);
        SolidityType::from(string)
    }};
}

pub fn address<T: Into<SolidityType>>(value: T) -> SolidityType {
    let value: SolidityType = Into::into(value);
    let address: Address = match value {
        SolidityType::Boolean(val) => {
            let val = val.to::<u8>();
            if val == 0 {
                ZERO_ADDRESS
            } else {
                Address::from_slice(&[1])
            }
        }
        SolidityType::Uint(val) => {
            let word = FixedBytes::from(val);
            Address::from_word(word)
        }
        SolidityType::Enum(val) => {
            let word = FixedBytes::with_last_byte(val.to());
            Address::from(word)
        }
        SolidityType::Address(val) => val,
        SolidityType::ByteArray(val) => {
            // NOTE I am not fully sure if this is correct.
            // The strings should be 0x prefixed. But not sure if that's always the case
            if val.len() <= 22 {
                let slice = &val[2..];
                Address::from_slice(&slice)
            } else {
                let slice = &val[2..22];
                Address::from_slice(&slice)
            }
        }
        SolidityType::FixedArray(val) => Address::from_word(val),
        SolidityType::String(val) => {
            if let Ok(address) = Address::from_str(&val) {
                address
            } else {
                return SolidityType::Null;
            }
        }
        SolidityType::Tuple(_)
        | SolidityType::List(_)
        | SolidityType::Struct(_)
        | SolidityType::Null => return SolidityType::Null,
    };

    SolidityType::Address(address)
}

pub fn string<T: Into<SolidityType>>(value: T) -> SolidityType {
    let value: SolidityType = Into::into(value);
    let string: String = match &value {
        SolidityType::Boolean(val) => {
            let val = val.to::<u8>();
            if val == 0 {
                "false".to_string()
            } else {
                "true".to_string()
            }
        }
        SolidityType::Uint(val) => val.to_string(),
        SolidityType::Enum(val) => val.to_string(),
        SolidityType::Address(val) => val.to_string(),
        SolidityType::ByteArray(val) => {
            // NOTE I am not fully sure if this is correct.
            // The strings should be 0x prefixed. But not sure if that's always the case
            val.to_string()
        }
        SolidityType::FixedArray(val) => val.to_string(),
        SolidityType::String(_) => {
            return value;
        }
        SolidityType::Tuple(_)
        | SolidityType::List(_)
        | SolidityType::Struct(_)
        | SolidityType::Null => return SolidityType::Null,
    };

    SolidityType::String(string)
}

pub fn uint<T: Into<SolidityType>>(value: T) -> SolidityType {
    let value: SolidityType = Into::into(value);
    let value = match value {
        SolidityType::Boolean(val) => {
            let val = val.to::<u8>();
            if val == 0 {
                Uint::from(0)
            } else {
                Uint::from(1)
            }
        }
        SolidityType::Uint(val) => val,
        SolidityType::Enum(val) => Uint::from(val),
        SolidityType::Address(val) => {
            let value = val.into_array();
            Uint::from_be_slice(&value[..])
        }
        SolidityType::ByteArray(val) => {
            // NOTE I am not fully sure if this is correct.
            // The strings should be 0x prefixed. But not sure if that's always the case
            let word = val.0;
            Uint::from_be_slice(&word[..])
        }
        SolidityType::FixedArray(val) => Uint::from_be_slice(&val.0),
        SolidityType::String(val) => {
            if let Ok(val) = val.parse() {
                val
            } else {
                return SolidityType::Null;
            }
        }
        SolidityType::Tuple(_)
        | SolidityType::List(_)
        | SolidityType::Struct(_)
        | SolidityType::Null => return SolidityType::Null,
    };

    SolidityType::Uint(value)
}

pub fn bytes<T: Into<SolidityType>>(value: T) -> SolidityType {
    let value: SolidityType = Into::into(value);
    let value = match value {
        SolidityType::Boolean(val) => {
            let val = val.to::<u8>();
            if val == 0 {
                Bytes::copy_from_slice(&[0])
            } else {
                Bytes::copy_from_slice(&[1])
            }
        }
        SolidityType::Uint(val) => {
            let bytes: [u8; 32] = val.to_be_bytes();
            Bytes::copy_from_slice(&bytes[..])
        }
        SolidityType::Enum(val) => {
            let byte: u8 = val.to::<u8>();
            Bytes::copy_from_slice(&[byte])
        }
        SolidityType::Address(val) => {
            let value = val.into_array();
            Bytes::copy_from_slice(&value[..])
        }
        SolidityType::ByteArray(val) => {
            // NOTE I am not fully sure if this is correct.
            // The strings should be 0x prefixed. But not sure if that's always the case
            let word = val.0;
            Bytes::copy_from_slice(&word[..])
        }
        SolidityType::FixedArray(val) => Bytes::copy_from_slice(&val.0),
        SolidityType::String(val) => {
            if let Ok(val) = val.parse() {
                val
            } else {
                return SolidityType::Null;
            }
        }
        SolidityType::Tuple(_)
        | SolidityType::List(_)
        | SolidityType::Struct(_)
        | SolidityType::Null => return SolidityType::Null,
    };

    SolidityType::ByteArray(value)
}
