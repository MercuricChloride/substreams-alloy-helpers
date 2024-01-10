/// This macro serves as a wrapper around the Alloy sol! macro
/// We derive From and To from the derive more crate so we can be more flexible with the type conversions
#[macro_export]
macro_rules! loose_sol {
    ($($body:tt)*) => {
        sol! {
            #[derive(::serde::Serialize, ::serde::Deserialize, Debug)]
            $($body)*
        }
    };
}

#[macro_export]
macro_rules! with_map {
    ($map_ident: ident ,$($body:tt)*) => {
        let mut $map_ident: SolidityType = SolidityType::Struct(HashMap::new());

        $($body)*

        // NOTE This is pretty slow, so I will speed this up eventually
        let maybe_value = $map_ident.to_maybe_value();
        match maybe_value {
            Some(val) => serde_json::from_value(val).unwrap(),
            None => None,
        }
    };
}

#[macro_export]
macro_rules! map_literal {
    ($($key: expr; $val: expr),*) => {{
        let mut map: SolidityType = SolidityType::Struct(HashMap::new());

        $(map.insert($key, $val);)*

        map
    }};
}

#[macro_export]
macro_rules! map_access {
    ($map:expr,$($key: expr),*) => {{
        let output = $map;
        $(let output = output.get($key).unwrap();)*
        output
    }};
}

#[macro_export]
macro_rules! map {
    ($value: expr, $callback: expr) => {
        $value.map($callback);
    };
}

#[macro_export]
macro_rules! filter {
    ($value: expr, $callback: expr) => {
        $value.filter($callback);
    };
}

/// A helper macro that allows us to convert any struct into a serde_json::Map
#[macro_export]
macro_rules! to_solidity_type {
    ($value: expr) => {
        serde_json::from_value::<SolidityType>(
            serde_json::to_value($value).expect("Failed to convert value into serde_json::Value"),
        )
        .expect("Failed to convert value into SolidityType")
    };
}

/// A helper macro that allows us to convert any map into an array
#[macro_export]
macro_rules! to_array {
    ($value: expr) => {
        if let Some(value) = $value {
            let as_value: serde_json::Value = serde_json::to_value(value).expect(
                "Couldn't convert value into serde_json Value in as_array! macro invocation",
            );
            match as_value {
                serde_json::Value::Array(arr) => Some(arr),
                _ => None,
            }
        } else {
            //substreams::log::println(format!("VALUE IS NONE!"));
            None
        }
    };
}

/// converts all inputs to a module that are a Struct protobuf into a Map with the same ident.
#[macro_export]
macro_rules! format_inputs {
    ($($input: ident),*) => {
        $(let $input = to_solidity_type!($input);)*
    };
}

/// A macro that allows us to convert a string, to a particular solidity type.
/// This is syntax sugar for parsing the string and wrapping in appropriate types
#[macro_export]
macro_rules! sol_type {
    ($variant: ident, $str: expr) => {
        SolidityType::$variant($str.parse().unwrap())
    };
}
