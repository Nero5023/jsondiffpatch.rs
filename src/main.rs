use json_diff_patch::diff_json;
use std::collections::HashMap;

fn main() {
    let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678",
                "xxx"
            ],
            "c": ["1","a", "c", "e", "f"]
        }"#;
    let data1 = r#"
        {
            "name": "John Doe bill",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ],
            "key0": "name1",
            "c": ["2", "3", "a", "b", "c", "d", "e"]
        }"#;
    let diffs = diff_json(data, data1);
    // TODO: check diffs is None
    let diffs = diffs.unwrap();

    // diff 2 map
    // TODO: make own path struct
    // let map = HashMap::new();

    println!("{:?}", diffs);
}
