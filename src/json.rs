use {crate::Litter, ::serde_json::Value as Json};

#[macro_export]
macro_rules! json {
    ($($tt:tt)+) => {
        $crate::Litter::inline(::serde_json::json!($($tt)+))
    };
}

pub fn json(source: &str) -> Litter<Json> {
    Litter::inline(serde_json::from_str(source).unwrap())
}

impl crate::Literal for Json {}
