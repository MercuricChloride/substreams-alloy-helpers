use crate::prelude::SolidityType;
use prost_wkt_types::Struct as ProtoStruct;
use substreams::prelude::*;

pub trait GenericStore<K, V> {
    fn generic_set(&self, key: K, value: V);

    fn generic_delete_prefix(&self, prefix: K);
}

impl GenericStore<&SolidityType, &SolidityType> for StoreSetProto<ProtoStruct> {
    fn generic_set(&self, key: &SolidityType, value: &SolidityType) {
        let key = key.to_string();
        let as_string = serde_json::to_string(value).unwrap();
        let as_value: ProtoStruct = serde_json::from_str(&as_string).unwrap();
        self.set(0, &key, &as_value);
    }

    fn generic_delete_prefix(&self, prefix: &SolidityType) {
        let prefix = prefix.to_string();

        self.delete_prefix(0, &prefix);
    }
}

impl GenericStore<SolidityType, SolidityType> for StoreSetProto<ProtoStruct> {
    fn generic_set(&self, key: SolidityType, value: SolidityType) {
        let key = key.to_string();
        let as_string = serde_json::to_string(&value).unwrap();
        let as_value: ProtoStruct = serde_json::from_str(&as_string).unwrap();
        self.set(0, &key, &as_value);
    }

    fn generic_delete_prefix(&self, prefix: SolidityType) {
        let prefix = prefix.to_string();

        self.delete_prefix(0, &prefix);
    }
}

impl GenericStore<&SolidityType, SolidityType> for StoreSetProto<ProtoStruct> {
    fn generic_set(&self, key: &SolidityType, value: SolidityType) {
        let key = key.to_string();
        let as_string = serde_json::to_string(&value).unwrap();
        let as_value: ProtoStruct = serde_json::from_str(&as_string).unwrap();
        self.set(0, &key, &as_value);
    }

    fn generic_delete_prefix(&self, prefix: &SolidityType) {
        let prefix = prefix.to_string();

        self.delete_prefix(0, &prefix);
    }
}

impl GenericStore<SolidityType, &SolidityType> for StoreSetProto<ProtoStruct> {
    fn generic_set(&self, key: SolidityType, value: &SolidityType) {
        let key = key.to_string();
        let as_string = serde_json::to_string(&value).unwrap();
        let as_value: ProtoStruct = serde_json::from_str(&as_string).unwrap();
        self.set(0, &key, &as_value);
    }

    fn generic_delete_prefix(&self, prefix: SolidityType) {
        let prefix = prefix.to_string();

        self.delete_prefix(0, &prefix);
    }
}

impl GenericStore<&SolidityType, &SolidityType> for StoreSetIfNotExistsProto<ProtoStruct> {
    fn generic_set(&self, key: &SolidityType, value: &SolidityType) {
        let key = key.to_string();
        let as_string = serde_json::to_string(value).unwrap();
        let as_value: ProtoStruct = serde_json::from_str(&as_string).unwrap();
        self.set_if_not_exists(0, &key, &as_value);
    }

    fn generic_delete_prefix(&self, prefix: &SolidityType) {
        let prefix = prefix.to_string();

        self.delete_prefix(0, &prefix);
    }
}

impl GenericStore<SolidityType, SolidityType> for StoreSetIfNotExistsProto<ProtoStruct> {
    fn generic_set(&self, key: SolidityType, value: SolidityType) {
        let key = key.to_string();
        let as_string = serde_json::to_string(&value).unwrap();
        let as_value: ProtoStruct = serde_json::from_str(&as_string).unwrap();
        self.set_if_not_exists(0, &key, &as_value);
    }

    fn generic_delete_prefix(&self, prefix: SolidityType) {
        let prefix = prefix.to_string();

        self.delete_prefix(0, &prefix);
    }
}

impl GenericStore<&SolidityType, SolidityType> for StoreSetIfNotExistsProto<ProtoStruct> {
    fn generic_set(&self, key: &SolidityType, value: SolidityType) {
        let key = key.to_string();
        let as_string = serde_json::to_string(&value).unwrap();
        let as_value: ProtoStruct = serde_json::from_str(&as_string).unwrap();
        self.set_if_not_exists(0, &key, &as_value);
    }

    fn generic_delete_prefix(&self, prefix: &SolidityType) {
        let prefix = prefix.to_string();

        self.delete_prefix(0, &prefix);
    }
}

impl GenericStore<SolidityType, &SolidityType> for StoreSetIfNotExistsProto<ProtoStruct> {
    fn generic_set(&self, key: SolidityType, value: &SolidityType) {
        let key = key.to_string();
        let as_string = serde_json::to_string(value).unwrap();
        let as_value: ProtoStruct = serde_json::from_str(&as_string).unwrap();
        self.set_if_not_exists(0, &key, &as_value);
    }

    fn generic_delete_prefix(&self, prefix: SolidityType) {
        let prefix = prefix.to_string();

        self.delete_prefix(0, &prefix);
    }
}
