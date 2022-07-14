use ::litter::{self, json, Litter};

#[test]
fn test() {
    let x = json!({a: 1});
    let y = json(r#"{"a": 1}"#);
}
