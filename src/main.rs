use core::result;
use serde_json::map::Map;
use serde_json::{Result, Value};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path;

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

fn read_json_file<P: AsRef<path::Path>>(path: P) -> result::Result<Value, Box<dyn Error>> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let v = serde_json::from_reader(reader)?;
    Ok(v)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PathElem {
    Key(String),
    Index(usize),
}

type Path = Vec<PathElem>;

// TODO: use reference not own
#[derive(Debug, PartialEq, Eq)]
struct DiffElem {
    diff: DiffChange,
    path: Path,
}

#[derive(Debug, PartialEq, Eq)]
enum DiffChange {
    Replace { old_val: Value, new_val: Value },
    Add(Value),
    Remove(Value),
}

fn diff_json(jval0: &Value, jval1: &Value, mut diffs: Vec<DiffElem>, path: Path) -> Vec<DiffElem> {
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
    path: Path,
) -> Vec<DiffElem> {
    for (k, v0) in m0.iter() {
        if let Some(v1) = m1.get(k) {
            let mut new_path = path.clone();
            new_path.push(PathElem::Key(k.to_string()));
            diffs = diff_json(v0, v1, diffs, new_path);
        }
    }
    let keys0: HashSet<String> = m0.keys().cloned().collect();
    let keys1: HashSet<String> = m1.keys().cloned().collect();
    let keys_only_in_m0 = keys0.difference(&keys1);
    let keys_only_in_m1 = keys1.difference(&keys0);

    for k in keys_only_in_m0 {
        let mut new_path = path.clone();
        new_path.push(PathElem::Key(k.to_string()));
        diffs.push(DiffElem {
            diff: DiffChange::Remove(m0.get(k).unwrap().clone()),
            path: new_path,
        })
    }

    for k in keys_only_in_m1 {
        let mut new_path = path.clone();
        new_path.push(PathElem::Key(k.to_string()));
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
    path: Path,
) -> Vec<DiffElem> {
    let len0 = arr0.len();
    let len1 = arr1.len();
    let min_len = len0.min(len1);
    for i in 0..min_len {
        let mut new_path = path.clone();
        new_path.push(PathElem::Index(i));
        diffs = diff_json(&arr0[i], &arr1[i], diffs, new_path);
    }
    if len0 >= len1 {
        let subv = &arr0[len1..];
        for (i, v) in subv.iter().enumerate() {
            let mut new_path = path.clone();
            let original_idx = i + len1;
            new_path.push(PathElem::Index(original_idx));
            diffs.push(DiffElem {
                diff: DiffChange::Remove(v.clone()),
                path: new_path,
            })
        }
    } else {
        let subv = &arr1[len0..];
        for (i, v) in subv.iter().enumerate() {
            let mut new_path = path.clone();
            let original_idx = i + len0;
            new_path.push(PathElem::Index(original_idx));
            diffs.push(DiffElem {
                diff: DiffChange::Add(v.clone()),
                path: new_path,
            })
        }
    }
    diffs
}

fn lcs<T: PartialEq + std::fmt::Debug>(arr0: &[T], arr1: &[T]) -> Vec<(usize, usize)> {
    let len0 = arr0.len();
    let len1 = arr1.len();
    let mut dp = vec![vec![0; len1 + 1]; len0 + 1];
    for (i, v0) in arr0.iter().enumerate() {
        for (j, v1) in arr1.iter().enumerate() {
            if v0 == v1 {
                dp[i + 1][j + 1] = dp[i][j] + 1;
            } else {
                dp[i + 1][j + 1] = dp[i + 1][j].max(dp[i][j + 1]);
            }
        }
    }

    let mut i = len0;
    let mut j = len1;
    let mut res = vec![];
    while i > 0 && j > 0 {
        if dp[i][j] == dp[i - 1][j] {
            i -= 1;
        } else if dp[i][j] == dp[i][j - 1] {
            j -= 1;
        } else {
            assert!(arr0[i - 1] == arr1[j - 1]);
            res.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        }
    }

    res.reverse();
    res
}

#[cfg(test)]
mod tests {
    use crate::diff_json;
    use crate::lcs;
    use crate::read_json_str;
    use crate::DiffChange;
    use crate::DiffElem;
    use crate::PathElem;
    use crate::Value;

    fn check_diff(original: &str, dest: &str, expect_diff: Vec<DiffElem>) {
        let origin_json = read_json_str(original);
        assert!(origin_json.is_ok(), "origin json not valid");
        let dest_json = read_json_str(dest);
        assert!(dest_json.is_ok(), "origin json not valid");
        let origin_json = origin_json.unwrap();
        let dest_json = dest_json.unwrap();
        let actual_diff = diff_json(&origin_json, &dest_json, vec![], vec![]);
        assert_eq!(actual_diff, expect_diff);
    }

    fn check_diff_same(original: &str, dest: &str) {
        check_diff(original, dest, vec![]);
    }

    #[test]
    fn test_bool() {
        let s0 = r#"{"x": true}"#;
        let s1 = r#"{"x": true}"#;
        check_diff_same(s0, s1);

        let s0 = r#"{"x": false}"#;
        let s1 = r#"{"x": false}"#;
        check_diff_same(s0, s1);

        let s0 = r#"{"x": true}"#;
        let s1 = r#"{"x": false}"#;

        let diff = DiffElem {
            diff: DiffChange::Replace {
                old_val: Value::Bool(true),
                new_val: Value::Bool(false),
            },
            path: vec![PathElem::Key("x".to_string())],
        };

        check_diff(s0, s1, vec![diff])
    }

    #[test]
    fn test_lcs() {
        let arr0: Vec<char> = "abcde".chars().collect();
        let arr1: Vec<char> = "ace".chars().collect();
        let res = lcs(&arr0, &arr1);
        assert_eq!(res, vec![(0, 0), (2, 1), (4, 2)]);
    }
}
