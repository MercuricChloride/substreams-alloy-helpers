use crate::aliases::*;
use alloy_primitives::{FixedBytes, Log};
use alloy_sol_types::SolEvent;
use serde::Serialize;
use serde_json::Value;
use substreams_ethereum::{block_view::LogView, pb::eth::v2::Block};

pub trait BlockHelpers {
    fn alloy_logs(&self, addresses: &[&Address]) -> Vec<Log>;
}

impl BlockHelpers for Block {
    fn alloy_logs(&self, addresses: &[&Address]) -> Vec<Log> {
        self.logs()
            .filter(|log| {
                if addresses.is_empty() {
                    true
                } else {
                    let address = Address::from_slice(log.address());
                    addresses.contains(&&address)
                }
            })
            .map(|l| l.into_log())
            .collect()
    }
}

pub trait JsonSolTypes {
    fn as_json(self) -> Value;
}

pub trait EventHelpers {
    fn get_events(blk: &Block, addresses: &[&Address]) -> Vec<prost_wkt_types::Struct>;
}

impl<T> EventHelpers for T
where
    T: SolEvent + Serialize + JsonSolTypes,
{
    fn get_events(blk: &Block, addresses: &[&Address]) -> Vec<prost_wkt_types::Struct> {
        let validate = false;
        blk.alloy_logs(addresses)
            .iter()
            .filter_map(|l| T::decode_log_object(l, validate).ok())
            .map(|e| e.as_json())
            .map(|e| serde_json::to_string_pretty(&e).unwrap())
            .map(|e| serde_json::from_str(&e).unwrap())
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
