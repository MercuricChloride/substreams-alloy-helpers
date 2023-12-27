/// This macro serves as a wrapper around the Alloy sol! macro
/// We derive From and To from the derive more crate so we can be more flexible with the type conversions
#[macro_export]
macro_rules! loose_sol {
    ($($body:tt)*) => {
        sol! {
            #[derive(::derive_more::From, ::serde::Serialize, ::serde::Deserialize, ::substreams_alloy_macros::JsonSolTypes)]
            $($body)*
        }
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
