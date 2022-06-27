use core::result;
use serde_json::map::Map;
use serde_json::{Result, Value};
use std::collections::HashSet;
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
                "+44 2345678",
                "xxx"
            ]
        }"#;
    let data1 = r#"
        {
            "name": "John Doe bill",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ],
            "key0": "name1"
        }"#;
    let json = read_json_str(data).unwrap();
    let json1 = read_json_str(data1).unwrap();
    let diffs = Vec::new();
    let diffs = diff_json(&json, &json1, diffs);
    println!("{:?}", diffs);
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

// TODO: use reference not own
#[derive(Debug)]
struct DiffElem {
    old_val: Value,
    new_val: Value,
}

fn diff_json(jval0: &Value, jval1: &Value, mut diffs: Vec<DiffElem>) -> Vec<DiffElem> {
    match (jval0, jval1) {
        (Value::Null, Value::Null) => diffs,
        (Value::Bool(b0), Value::Bool(b1)) if b0 == b1 => diffs,
        (Value::Number(n0), Value::Number(n1)) if n0 == n1 => diffs,
        (Value::String(s0), Value::String(s1)) if s0 == s1 => diffs,
        (Value::Object(m0), Value::Object(m1)) => diff_json_map(m0, m1, diffs),
        (Value::Array(v0), Value::Array(v1)) => diff_json_arr(v0.as_slice(), v1.as_slice(), diffs),
        (a0, a1) => {
            diffs.push(DiffElem {
                old_val: jval0.clone(),
                new_val: jval1.clone(),
            });
            diffs
        }
        (_, _) => diffs,
    }
}

fn diff_json_map(
    m0: &Map<String, Value>,
    m1: &Map<String, Value>,
    mut diffs: Vec<DiffElem>,
) -> Vec<DiffElem> {
    for (k, v0) in m0.iter() {
        if let Some(v1) = m1.get(k) {
            diffs = diff_json(v0, v1, diffs)
        }
    }
    let keys0: HashSet<String> = m0.keys().cloned().collect();
    let keys1: HashSet<String> = m1.keys().cloned().collect();
    let keys_only_in_m0 = keys0.difference(&keys1);
    let keys_only_in_m1 = keys1.difference(&keys0);

    for k in keys_only_in_m0 {
        diffs.push(DiffElem {
            old_val: m0.get(k).unwrap().clone(),
            new_val: Value::Null,
        })
    }

    for k in keys_only_in_m1 {
        diffs.push(DiffElem {
            old_val: Value::Null,
            new_val: m1.get(k).unwrap().clone(),
        })
    }
    diffs
}

fn diff_json_arr(v0: &[Value], v1: &[Value], mut diffs: Vec<DiffElem>) -> Vec<DiffElem> {
    let len0 = v0.len();
    let len1 = v1.len();
    let min_len = len0.min(len1);
    for i in 0..min_len {
        diffs = diff_json(&v0[i], &v1[i], diffs);
    }
    if len0 >= len1 {
        let subv = &v0[len1..];
        println!("subv0 len0 {} len1 {} {:?}", len0, len1, subv);
        for v in subv.iter() {
            diffs.push(DiffElem {
                old_val: v.clone(),
                new_val: Value::Null,
            })
        }
    } else {
        let subv = &v1[len0..];
        for v in subv.iter() {
            diffs.push(DiffElem {
                old_val: Value::Null,
                new_val: v.clone(),
            })
        }
    }
    diffs
}
