use std::str::FromStr;

use alloy_primitives::*;

use crate::prelude::*;

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
                "true".to_string()
            } else {
                "false".to_string()
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
    let string: String = match &value {
        SolidityType::Boolean(val) => {
            let val = val.to::<u8>();
            if val == 0 {
                "true".to_string()
            } else {
                "false".to_string()
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
        SolidityType::FixedArray(_) => todo!(),
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
