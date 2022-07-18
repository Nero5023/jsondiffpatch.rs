use json_diff_patch::diff_json;
use json_diff_patch::read_json_str;

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
    let json = read_json_str(data).unwrap();
    let json1 = read_json_str(data1).unwrap();
    let diffs = Vec::new();
    let diffs = diff_json(&json, &json1, diffs, vec![]);
    println!("{:?}", diffs);
}
