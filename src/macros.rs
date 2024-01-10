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

        match serde_json::to_value($map_ident).ok()? {
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    None
                } else {
                    let mut map: Map<String, Value> = Map::new();
                    map.insert("values".to_string(), serde_json::to_value(arr).unwrap());
                    Some(serde_json::from_value(serde_json::to_value(map).unwrap()).unwrap())
                }
            },
            serde_json::Value::Object(obj) => {
                if obj.is_empty() {
                    None
                } else {
                    Some(serde_json::from_value(serde_json::to_value(obj).unwrap()).unwrap())
                }
            },
            serde_json::Value::Null => None,
            _ => panic!("Not sure how to convert this type into a map for output!")
        }
    };
    // ($map_ident: ident ,$($body:tt)*) => {
    //     let mut $map_ident: serde_json::Map<_, Value> = Map::new();

    //     $($body)*

    //     let $map_ident = match serde_json::to_value($map_ident).ok()? {
    //         serde_json::Value::Array(arr) => {
    //             if arr.is_empty() {
    //                 None
    //             } else {
    //                 Some(map_literal!("values"; arr))
    //             }
    //         },
    //         serde_json::Value::Object(obj) => {
    //             if obj.is_empty() {
    //                 None
    //             } else {
    //                 Some(serde_json::to_value(obj).ok())
    //             }
    //         },
    //         serde_json::Value::Null => None,
    //         _ => panic!("Not sure how to convert this type into a map for output!")
    //     };

    //     if let Some(Some(val)) = $map_ident {
    //         serde_json::from_value(val).ok()?
    //     } else {
    //         None
    //     }
    // };
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
