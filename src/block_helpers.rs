use crate::{
    aliases::*,
    parse_as,
    prelude::{format_hex, SolidityJsonValue, SolidityType},
    sol_type,
};
use alloy_primitives::{FixedBytes, Log};
use alloy_sol_types::{SolCall, SolEnum, SolEvent};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use substreams_ethereum::{block_view::LogView, pb::eth::v2::Block};

#[derive(Serialize, Deserialize)]
pub struct TxMeta {
    from: SolidityJsonValue,
    to: SolidityJsonValue,
    block_number: SolidityJsonValue,
}

impl TxMeta {
    pub fn new(from: &String, to: &String, block_number: &String) -> TxMeta {
        TxMeta {
            from: sol_type!(Address, from),
            to: sol_type!(Address, to),
            block_number: sol_type!(Uint, block_number),
        }
    }

    fn from_log(value: &substreams_ethereum::block_view::LogView, block_number: &String) -> Self {
        let txn = &value.receipt.transaction;
        let (from, to) = (&format_hex(&txn.from), &format_hex(&txn.to));
        TxMeta::new(from, to, block_number)
    }
}

pub trait BlockHelpers {
    fn alloy_logs(&self, addresses: &[&Address]) -> Vec<(Log, TxMeta)>;
}

impl BlockHelpers for Block {
    fn alloy_logs(&self, addresses: &[&Address]) -> Vec<(Log, TxMeta)> {
        let block_number = self.number.to_string();
        self.logs()
            .filter_map(|log| {
                if addresses.is_empty() {
                    Some((log, TxMeta::from_log(&log, &block_number)))
                } else {
                    let address = Address::from_slice(log.address());
                    if addresses.contains(&&address) {
                        Some((log, TxMeta::from_log(&log, &block_number)))
                    } else {
                        None
                    }
                }
            })
            .map(|(log, meta)| (log.into_log(), meta))
            .collect()
    }
}

pub trait EventHelpers {
    fn get_events(blk: &Block, addresses: &[&Address]) -> Vec<prost_wkt_types::Struct>;
}

impl<T> EventHelpers for T
where
    T: SolEvent + Serialize,
{
    fn get_events(blk: &Block, addresses: &[&Address]) -> Vec<prost_wkt_types::Struct> {
        let validate = false;
        blk.alloy_logs(addresses)
            .iter()
            .filter_map(|(l, meta)| {
                let event = T::decode_log_object(l, validate).ok();
                if let Some(event) = event {
                    Some((event, meta))
                } else {
                    None
                }
            })
            .map(|(event, meta)| {
                let map = serde_json::to_value(event).unwrap();
                let event_guess = SolidityJsonValue::guess_json_value(&map).unwrap();
                let as_value = serde_json::to_value(event_guess).unwrap();
                if let Value::Object(mut map) = as_value {
                    let key = String::from("tx_meta");

                    if map.get(&key).is_some() {
                        panic!("Map contains the tx_meta key already!");
                    }

                    if let Some(Value::Object(map)) = map.get_mut("value") {
                        map.insert(key, serde_json::to_value(meta).unwrap());
                    } else {
                        panic!("Couldn't insert tx_meta into Struct value field!");
                    }

                    serde_json::from_value(map.into())
                        .expect("Failed convert event into protobuf struct")
                } else {
                    panic!("Event wasn't found to be a serde_json::Value Object!?");
                }
            })
            .collect()
    }
}

pub trait FunctionHelpers {
    fn rpc_call(&self) -> prost_wkt_types::Struct;
}

pub trait AlloyLog {
    fn into_log(&self) -> Log;
}

impl AlloyLog for LogView<'_> {
    fn into_log(&self) -> Log {
        let topics = self
            .topics()
            .iter()
            .map(|t| FixedBytes::try_from(&t.as_slice()[..]).unwrap())
            .collect();

        let data = self.data().to_vec().into();

        Log::new(topics, data).expect("Couldn't create a AlloyLog from a LogView")
    }
}
