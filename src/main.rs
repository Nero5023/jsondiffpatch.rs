use core::result;
use serde_json::{Result, Value};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() {
    println!("Hello, world!");
    let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;
    let json = read_json_str(data);
    pattern_match(&json.unwrap())
}

fn read_json_str(s: &str) -> Result<Value> {
    let v: Value = serde_json::from_str(s)?;
    println!("{}", v);
    Ok(v)
}

fn read_json_file<P: AsRef<Path>>(path: P) -> result::Result<Value, Box<dyn Error>> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let v = serde_json::from_reader(reader)?;
    Ok(v)
}

fn pattern_match(v: &Value) {
    match v {
        Value::Null => println!("null"),
        Value::Bool(b) => println!("bool {}", b),
        Value::Array(arr) => arr.iter().for_each(pattern_match),
        Value::Number(n) => println!("number {}", n),
        Value::String(s) => println!("string {}", s),
        Value::Object(o) => {
            println!("map");
            o.values().for_each(pattern_match);
        }
    }
}
