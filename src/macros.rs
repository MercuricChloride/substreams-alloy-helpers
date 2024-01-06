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

        //substreams::log::println(format!("\n\nmap_ident value: \n\n{:?}", $map_ident));
        let $map_ident = match serde_json::to_value($map_ident).ok()? {
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    None
                } else {
                    Some(map_literal!("values"; arr))
                }
            },
            serde_json::Value::Object(obj) => {
                if obj.is_empty() {
                    None
                } else {
                    Some(serde_json::to_value(obj).ok())
                }
            },
            serde_json::Value::Null => None,
            _ => panic!("Not sure how to convert this type into a map for output!")
        };

        if let Some(Some(val)) = $map_ident {
            serde_json::from_value(val).ok()?
        } else {
            None
        }
    };
}

#[macro_export]
macro_rules! map_insert {
    ($key: expr, $val: expr, $map_ident: ident) => {
        let mut should_insert_val = false;
        let val = serde_json::to_value($val).ok();
        if let Some(val) = &val {
            match val {
                serde_json::Value::Array(arr) => {
                    if !arr.is_empty() {
                        should_insert_val = true;
                    }
                }
                serde_json::Value::Object(obj) => {
                    if !obj.is_empty() {
                        should_insert_val = true;
                    }
                }
                serde_json::Value::Null => {}
                _ => {
                    should_insert_val = true;
                }
            }
        };

        if should_insert_val {
            $map_ident.insert($key.to_string(), val.unwrap())
        } else {
            None
        }
    };
}

#[macro_export]
macro_rules! map_literal {
    ($($key: expr; $val: expr),*) => {{
        let mut output_map: serde_json::Map<_, Value> = Map::new();

        $(map_insert!($key, $val, output_map);)*

        serde_json::to_value(output_map).ok()
    }};
}

#[macro_export]
macro_rules! map_access {
    ($map:expr,$($key: expr),*) => {{
        let as_value = serde_json::to_value($map).ok()?;
        let output: serde_json::Map<String, Value> = serde_json::from_value(as_value).ok()?;
        let output: serde_json::Map<String, Value> = match output.get("kind") {
            Some(serde_json::Value::String(s)) => match s {
                s if s.starts_with("tuple") => {
                    let value = output.get("value").expect("A tuple should always have a value field").clone();
                    if let serde_json::Value::Array(arr) = value {
                        let arr = arr.into_iter().enumerate().map(|(index, value)| (index.to_string(), value));
                        serde_json::Map::<String, Value>::from_iter(arr)
                    } else {
                        panic!("Tuple value field not an array!")
                    }
                    //serde_json::from_value().expect("Couldn't serialize tuple into Map")
                }
                s if s.starts_with("list") => {
                    let value = output.get("value").expect("A list should always have a value field").clone();
                    if let serde_json::Value::Array(arr) = value {
                        let arr = arr.into_iter().enumerate().map(|(index, value)| (index.to_string(), value));
                        serde_json::Map::<String, Value>::from_iter(arr)
                    } else {
                        panic!("List value field not an array!")
                    }
                    //serde_json::from_value(output.get("value").expect("A list should always have a value field").clone()).expect("Couldn't serialize list into Map")
                }
                _ => panic!("{:?}", s)
            },
            None => output,
            _ => panic!("Trying to use a scalar type as a map! Don't do this pls, it's a logical error!")
        };
        substreams::log::println(format!("{:?}", &output));
        $(let output = output.get($key)?;)*
        Some(output.clone())
    }};
}

/// A helper macro that allows us to convert any struct into a serde_json::Map
#[macro_export]
macro_rules! map {
    ($value: expr, $callback: expr) => {
        if let Some(val) = $value.as_ref() {
            match serde_json::to_value(val).unwrap() {
                serde_json::Value::Array(arr) => Some(
                    arr.into_iter()
                        .map($callback)
                        .filter_map(|element| serde_json::to_value(element).ok())
                        .collect(),
                ),
                serde_json::Value::Object(obj) => {
                    if let Some(serde_json::Value::Array(arr)) = obj.get("values").as_ref() {
                        Some(
                            arr.into_iter()
                                .map($callback)
                                .filter_map(|element| serde_json::to_value(element).ok())
                                .collect(),
                        )
                    } else {
                        None
                    }
                }
                _ => {
                    substreams::log::println(format!(
                        "Failed quietly, couldn't map over null or not an array! {:?}",
                        $value
                    ));
                    None
                }
            }
        } else {
            None
        };
    };
}

#[macro_export]
macro_rules! filter {
    ($value: expr, $callback: expr) => {
        if let Some(val) = $value.as_ref() {
            match val {
                serde_json::Value::Array(arr) => Some(
                    arr.into_iter()
                        .filter($callback)
                        .map(|element| serde_json::to_value(element).unwrap())
                        .collect(),
                ),
                serde_json::Value::Object(obj) => {
                    if let Some(serde_json::Value::Array(arr)) = obj.get("values").as_ref() {
                        Some(
                            arr.into_iter()
                                .filter($callback)
                                .map(|element| serde_json::to_value(element).unwrap())
                                .collect(),
                        )
                    } else {
                        None
                    }
                }
                _ => {
                    substreams::log::println(format!(
                        "Failed quietly, couldn't filter over null or not an array! {:?}",
                        $value
                    ));
                    None
                }
            }
        } else {
            //substreams::log::println(format!(
            //"Failed quietly, couldn't filter over null or not an array! {:?}",
            //$value
            //));
            None
        };
    };
}

/// A helper macro that allows us to convert any struct into a serde_json::Map
#[macro_export]
macro_rules! to_map {
    ($value: expr) => {
        serde_json::from_value::<serde_json::Map<_, serde_json::Value>>(
            serde_json::to_value($value).expect("hi"),
        )
        .expect("hello")
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
        $(let $input = to_map!($input);)*
    };
}

/// Just a simple wrapper that adds syntax sugar for parsing our custom json value type
#[macro_export]
macro_rules! parse_as {
    ($self: ident, $variant: ident) => {
        match $self.value {
            ValueKind::Scalar(val) => SolidityType::$variant(val.parse().unwrap()),
            ValueKind::Compound(val) => {
                SolidityType::Tuple(val.into_iter().map(|item| item.to_sol_type()).collect())
            }
        }
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
