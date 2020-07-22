mod encode;
pub use encode::Json;

#[macro_export]
macro_rules! jsons {
    ($e:tt) => { json!($e).to_string() }
}

#[macro_export]
macro_rules! json {
    (null) => { json::Json::Null };

    ([$($e:tt),*]) => {
        json::Json::Array(vec![
            $(
            json::json!($e),
            )*
        ])
    };

    ([$($e:tt,)*]) => { json!([$($e),*]) };

    ({$($name:ident: $e:tt),*}) => {{
        let mut map = std::collections::HashMap::new();

        $(
        map.insert(stringify!($name).into(), json::json!($e));
        )*

        json::Json::Object(map)
    }};

    ({$($name:ident: $e:tt,)*}) => { json::json!({$($name: $e),*}) };

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

impl Jsonable for usize {
    fn into_json(self) -> Json { Json::Number(self as f64) }
}