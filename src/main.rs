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
    let diffs = diff_json(&json, &json1, diffs, vec![]);
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

// TODO: use reference not own
#[derive(Debug)]
struct DiffElem {
    diff: DiffChange,
    path: Vec<String>,
}

#[derive(Debug)]
enum DiffChange {
    Replace { old_val: Value, new_val: Value },
    Add(Value),
    Remove(Value),
}

fn diff_json(
    jval0: &Value,
    jval1: &Value,
    mut diffs: Vec<DiffElem>,
    path: Vec<String>,
) -> Vec<DiffElem> {
    match (jval0, jval1) {
        (Value::Null, Value::Null) => diffs,
        (Value::Bool(b0), Value::Bool(b1)) if b0 == b1 => diffs,
        (Value::Number(n0), Value::Number(n1)) if n0 == n1 => diffs,
        (Value::String(s0), Value::String(s1)) if s0 == s1 => diffs,
        (Value::Object(m0), Value::Object(m1)) => diff_json_map(m0, m1, diffs, path),
        (Value::Array(v0), Value::Array(v1)) => {
            diff_json_arr(v0.as_slice(), v1.as_slice(), diffs, path)
        }
        (_, _) => {
            // not equal case
            diffs.push(DiffElem {
                diff: DiffChange::Replace {
                    old_val: jval0.clone(),
                    new_val: jval1.clone(),
                },
                path: path,
            });
            diffs
        }
    }
}

fn diff_json_map(
    m0: &Map<String, Value>,
    m1: &Map<String, Value>,
    mut diffs: Vec<DiffElem>,
    path: Vec<String>,
) -> Vec<DiffElem> {
    for (k, v0) in m0.iter() {
        if let Some(v1) = m1.get(k) {
            let mut new_path = path.clone();
            new_path.push(k.to_string());
            diffs = diff_json(v0, v1, diffs, new_path);
        }
    }
    let keys0: HashSet<String> = m0.keys().cloned().collect();
    let keys1: HashSet<String> = m1.keys().cloned().collect();
    let keys_only_in_m0 = keys0.difference(&keys1);
    let keys_only_in_m1 = keys1.difference(&keys0);

    for k in keys_only_in_m0 {
        let mut new_path = path.clone();
        new_path.push(k.to_string());
        diffs.push(DiffElem {
            diff: DiffChange::Remove(m0.get(k).unwrap().clone()),
            path: new_path,
        })
    }

    for k in keys_only_in_m1 {
        let mut new_path = path.clone();
        new_path.push(k.to_string());
        diffs.push(DiffElem {
            diff: DiffChange::Add(m1.get(k).unwrap().clone()),
            path: new_path,
        })
    }

    diffs
}

fn diff_json_arr(
    arr0: &[Value],
    arr1: &[Value],
    mut diffs: Vec<DiffElem>,
    path: Vec<String>,
) -> Vec<DiffElem> {
    let len0 = arr0.len();
    let len1 = arr1.len();
    let min_len = len0.min(len1);
    for i in 0..min_len {
        diffs = diff_json(&arr0[i], &arr1[i], diffs, path.clone());
    }
    if len0 >= len1 {
        let subv = &arr0[len1..];
        for v in subv.iter() {
            diffs.push(DiffElem {
                diff: DiffChange::Remove(v.clone()),
                path: path.clone(),
            })
        }
    } else {
        let subv = &arr1[len0..];
        for v in subv.iter() {
            diffs.push(DiffElem {
                diff: DiffChange::Add(v.clone()),
                path: path.clone(),
            })
        }
    }
    diffs
}
