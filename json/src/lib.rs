pub mod json;

pub use json::Json;
// pub use json_macro::Jsonable;

#[macro_export]
macro_rules! jsons {
    ($e:tt) => { jsont!($e).to_string() }
}

#[macro_export]
macro_rules! jsont {
    (null) => { json::Json::Null };

    ([$($e:tt),*]) => {
        json::Json::Array(vec![
        $(
            json::jsont!($e),
        )*
        ])
    };

    ([$($e:tt,)*]) => { jsont!([$($e),*]) };

    ({$($name:ident: $e:tt),*}) => {{
        let mut map = std::collections::HashMap::new();

        $(
        map.insert(stringify!($name).into(), json::jsont!($e));
        )*

        json::Json::Object(map)
    }};

    ({$($name:ident: $e:tt,)*}) => { json::jsont!({$($name: $e),*}) };

    ($e:expr) => { json::Jsonable::into_json($e) };

}

pub trait Jsonable {
    fn into_json(self) -> Json;
}

impl Jsonable for Json {
    fn into_json(self) -> Json { self }
}

impl Jsonable for bool {
    fn into_json(self) -> Json { Json::Boolean(self) }
}

impl Jsonable for &str {
    fn into_json(self) -> Json { Json::String(self.into()) }
}

impl Jsonable for String {
    fn into_json(self) -> Json { Json::String(self) }
}

impl Jsonable for f64 {
    fn into_json(self) -> Json { Json::Number(self) }
}

impl Jsonable for i32 {
    fn into_json(self) -> Json { Json::Number(self as f64) }
}

impl Jsonable for u8 {
    fn into_json(self) -> Json { Json::Number(self as f64) }
}