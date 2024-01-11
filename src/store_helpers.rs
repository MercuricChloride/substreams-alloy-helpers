use crate::prelude::SolidityType;
use prost_wkt_types::Struct as ProtoStruct;
use serde::Serialize;
use substreams::prelude::*;

pub trait GenericStore<K, V> {
    fn generic_set(&self, key: K, value: V);

    fn generic_delete_prefix(&self, prefix: K);
}

impl<K, V> GenericStore<K, V> for StoreSetProto<ProtoStruct>
where
    K: AsRef<SolidityType> + ToString,
    V: AsRef<SolidityType> + Serialize,
{
    fn generic_set(&self, key: K, value: V) {
        let key = key.to_string();
        let as_string = serde_json::to_string(&value).unwrap();
        let as_value: ProtoStruct = serde_json::from_str(&as_string).unwrap();
        self.set(0, &key, &as_value);
    }

    fn generic_delete_prefix(&self, prefix: K) {
        let prefix = prefix.to_string();
        self.delete_prefix(0, &prefix);
    }
}

impl<K, V> GenericStore<K, V> for StoreSetIfNotExistsProto<ProtoStruct>
where
    K: AsRef<SolidityType> + ToString,
    V: AsRef<SolidityType> + Serialize,
{
    fn generic_set(&self, key: K, value: V) {
        let key = key.to_string();
        let as_string = serde_json::to_string(&value).unwrap();
        let as_value: ProtoStruct = serde_json::from_str(&as_string).unwrap();
        self.set_if_not_exists(0, &key, &as_value);
    }

    fn generic_delete_prefix(&self, prefix: K) {
        let prefix = prefix.to_string();
        self.delete_prefix(0, &prefix);
    }
}
