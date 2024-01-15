//! The purpose of this module is to allow us to have dynamically typed local variables in our substreams modules

use crate::json_values::*;
use crate::prelude::GenericStoreGet;
use prost_wkt_types::Struct;
use substreams::store::StoreGetProto;

macro_rules! only {
    ($value: expr, $variant: ident, $error_msg: expr) => {
        if let LocalVar::$variant(val) = $value {
            val
        } else {
            panic!($error_msg)
        }
    };
}

macro_rules! only_store {
    ($value: expr) => {
        only!(
            $value,
            StoreGet,
            "Tried to use a solidity type as a store module in get mode. Please don't do this!"
        )
    };
}

macro_rules! only_sol {
    ($value: expr) => {
        only!($value, SolidityType, "Tried to use a store in get mode as a solidity value. Please don't do this! Use the get function to access values inside!")
    };
}

pub enum LocalVar {
    /// A local variable that is a literal value
    SolidityType(SolidityType),
    /// A local variable of a store
    StoreGet(StoreGetProto<Struct>),
}

impl From<LocalVar> for SolidityType {
    fn from(value: LocalVar) -> Self {
        only_sol!(value)
    }
}

impl From<LocalVar> for StoreGetProto<Struct> {
    fn from(value: LocalVar) -> Self {
        only_store!(value)
    }
}

impl<K> GenericStoreGet<K> for LocalVar
where
    K: AsRef<SolidityType> + ToString,
{
    fn generic_get(&self, key: K) -> SolidityType {
        let value = only_store!(&self);
        value.generic_get(key)
    }
}
