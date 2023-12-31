/// This macro serves as a wrapper around the Alloy sol! macro
/// We derive From and To from the derive more crate so we can be more flexible with the type conversions
#[macro_export]
macro_rules! loose_sol {
    ($($body:tt)*) => {
        sol! {
            #[derive(::serde::Serialize, ::serde::Deserialize, ::substreams_alloy_macros::JsonSolTypes)]
            $($body)*
        }
    };
}

#[macro_export]
macro_rules! with_map {
    ($map_ident: ident ,$($body:tt)*) => {
        let mut $map_ident: serde_json::Map<_, Value> = Map::new();

        $($body)*

        serde_json::from_value(serde_json::to_value($map_ident).unwrap()).unwrap()
    };
}

#[macro_export]
macro_rules! map_insert {
    ($key: expr, $val: expr, $map_ident: ident) => {
        $map_ident.insert($key.to_string(), serde_json::to_value($val).unwrap());
    };
}

#[macro_export]
macro_rules! map_literal {
    ($($key: expr; $val: expr),*) => {{
        let mut output_map: serde_json::Map<_, Value> = Map::new();

        $(map_insert!($key, $val, output_map);)*

        serde_json::to_value(output_map).unwrap()
    }};
}

#[macro_export]
macro_rules! map_access {
    ($map:expr,$($key: expr),*) => {{
        let output = $map;
        $(let output = output.get($key).unwrap();)*
        output.clone()
    }};
}

/// A helper macro that allows us to convert any struct into a serde_json::Map
#[macro_export]
macro_rules! to_map {
    ($value: expr) => {
        serde_json::from_value::<serde_json::Map<_, serde_json::Value>>(
            serde_json::to_value($value).unwrap(),
        )
        .unwrap()
    };
}

/// A helper macro that allows us to convert any map into an array
#[macro_export]
macro_rules! to_array {
    ($value: expr) => {
        serde_json::from_str::<Vec<serde_json::Value>>(&serde_json::to_string(&$value).unwrap())
            .unwrap()
    };
}

/// converts all inputs to a module that are a Struct protobuf into a Map with the same ident.
#[macro_export]
macro_rules! format_inputs {
    ($($input: ident),*) => {
        $(let $input = to_map!($input);)*
    };
}

/// Just a simple wrapper that adds syntax sugar for parsing our custom json value type
#[macro_export]
macro_rules! parse_as {
    ($self: ident, $variant: ident) => {
        SolidityType::$variant($self.value.parse().unwrap())
    };
}

/// A macro that allows us to convert a string, to a particular solidity type.
/// This is syntax sugar for parsing the string and wrapping in appropriate types
#[macro_export]
macro_rules! sol_type {
    ($variant: ident, $str: expr) => {
        SolidityType::$variant($str.parse().unwrap()).to_json_value()
    };
}
