mod lcs;

use core::result;
use serde_json::map::Map;
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::ops::DerefMut;
use std::string::ToString;

fn read_json_str(s: &str) -> Result<Value> {
    let v: Value = serde_json::from_str(s)?;
    Ok(v)
}

fn read_json_file<P: AsRef<std::path::Path>>(path: P) -> result::Result<Value, Box<dyn Error>> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let v = serde_json::from_reader(reader)?;
    Ok(v)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum PathElem {
    Key(String),
    Index(usize),
}

impl Display for PathElem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathElem::Key(s) => {
                if s.starts_with('_') {
                    write!(f, "\\{}", s)
                } else {
                    write!(f, "{}", s)
                }
            }
            PathElem::Index(idx) => write!(f, "_{}", idx),
        }
    }
}

//type Path = Vec<PathElem>;
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Path(Vec<PathElem>);

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = String::from("/")
            + &self
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join("/");
        write!(f, "{}", s)
    }
}

impl Deref for Path {
    type Target = Vec<PathElem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Path {
    pub fn empty() -> Self {
        Self(vec![])
    }

    fn new(path: Vec<PathElem>) -> Self {
        Self(path)
    }

    pub fn parent_path(&self) -> Option<Self> {
        if self.len() == 0 {
            None
        } else {
            let mut path = self.clone();
            path.pop();
            Some(path)
        }
    }

    pub fn clone_then_add_key<T: ToString>(&self, s: T) -> Self {
        let mut new_path = self.clone();
        new_path.push(PathElem::Key(s.to_string()));
        new_path
    }

    pub fn clone_then_add_idx(&self, idx: usize) -> Self {
        let mut new_path = self.clone();
        new_path.push(PathElem::Index(idx));
        new_path
    }

    pub fn push_idx(&mut self, idx: usize) {
        self.push(PathElem::Index(idx))
    }

    pub fn push_key<T: ToString>(&mut self, s: T) {
        self.push(PathElem::Key(s.to_string()))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DiffElem {
    diff: DiffChange,
    path: Path,
}

impl Display for DiffElem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self.path.to_string();
        let mut diff_jmap = serde_json::Map::new();
        diff_jmap.insert("path".to_owned(), Value::String(path));
        match &self.diff {
            DiffChange::Add(val) => {
                diff_jmap.insert("new_val".to_owned(), val.clone());
                diff_jmap.insert("diff".to_owned(), Value::String("add".to_owned()));
            }
            DiffChange::Remove(val) => {
                diff_jmap.insert("old_val".to_owned(), val.clone());
                diff_jmap.insert("diff".to_owned(), Value::String("remove".to_owned()));
            }
            DiffChange::Replace { old_val, new_val } => {
                diff_jmap.insert("old_val".to_owned(), old_val.clone());
                diff_jmap.insert("new_val".to_owned(), new_val.clone());
                diff_jmap.insert("diff".to_owned(), Value::String("replace".to_owned()));
            }
        };
        let diff_j = Value::Object(diff_jmap);
        let pretty_diff_str = serde_json::to_string_pretty(&diff_j).unwrap();
        write!(f, "{}", pretty_diff_str)
    }
}

impl DiffElem {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn path_str(&self) -> String {
        self.path.to_string()
    }

    pub fn diff_change(&self) -> &DiffChange {
        &self.diff
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DiffChange {
    Replace { old_val: Value, new_val: Value },
    Add(Value),
    Remove(Value),
}

pub struct JsonDiff {
    path2change: HashMap<Path, DiffChange>,
    child_added_keys: HashMap<Path, Vec<String>>,
}

impl JsonDiff {
    fn new(diffs: Vec<DiffElem>) -> Self {
        let mut path2change = HashMap::new();
        let mut child_added_keys = HashMap::new();
        for diff in diffs {
            path2change.insert(diff.path().clone(), diff.diff_change().clone());
            if let DiffChange::Add(_) = diff.diff_change() {
                let last_key = diff.path().last().unwrap();
                // just for PathElem::Key
                if let PathElem::Key(str_key) = last_key {
                    let parent_path = diff.path().parent_path().unwrap();
                    let keys = child_added_keys.entry(parent_path).or_insert_with(Vec::new);
                    keys.push(str_key.to_owned());
                }
            }
        }
        Self {
            path2change,
            child_added_keys,
        }
    }

    pub fn diff_json(s0: &str, s1: &str) -> Result<Self> {
        let diffs = diff_json(s0, s1)?;
        Ok(Self::new(diffs))
    }

    pub fn get_add_keys(&self, path: &Path) -> Option<&Vec<String>> {
        self.child_added_keys.get(path)
    }

    pub fn get_diffchange(&self, path: &Path) -> Option<&DiffChange> {
        self.path2change.get(path)
    }
}

enum ArrDiffMethod {
    Simple,
    Lcs,
}

impl ArrDiffMethod {
    fn diff(
        &self,
        arr0: &[Value],
        arr1: &[Value],
        mut diffs: Vec<DiffElem>,
        path: Path,
    ) -> Vec<DiffElem> {
        match *self {
            ArrDiffMethod::Simple => {
                if arr0 == arr1 {
                    diffs
                } else {
                    diffs.push(DiffElem {
                        diff: DiffChange::Replace {
                            old_val: Value::Array(arr0.to_vec()),
                            new_val: Value::Array(arr1.to_vec()),
                        },
                        path,
                    });
                    diffs
                }
            }
            ArrDiffMethod::Lcs => diff_json_arr_lcs(arr0, arr1, diffs, path),
        }
    }
}

fn diff_json(json0: &str, json1: &str) -> Result<Vec<DiffElem>> {
    diff_json_str(json0, json1, ArrDiffMethod::Lcs)
}

fn diff_json_simple(json0: &str, json1: &str) -> Result<Vec<DiffElem>> {
    diff_json_str(json0, json1, ArrDiffMethod::Simple)
}

fn diff_json_str(json0: &str, json1: &str, arr_diff: ArrDiffMethod) -> Result<Vec<DiffElem>> {
    let path = Path::empty();
    let diffs = Vec::new();
    let json0 = read_json_str(json0)?;
    let json1 = read_json_str(json1)?;
    let diffs = diff_json_inner(&json0, &json1, diffs, path, &arr_diff);
    Ok(diffs)
}

fn diff_json_inner(
    jval0: &Value,
    jval1: &Value,
    mut diffs: Vec<DiffElem>,
    path: Path,
    arr_diff: &ArrDiffMethod,
) -> Vec<DiffElem> {
    match (jval0, jval1) {
        (Value::Null, Value::Null) => diffs,
        (Value::Bool(b0), Value::Bool(b1)) if b0 == b1 => diffs,
        (Value::Number(n0), Value::Number(n1)) if n0 == n1 => diffs,
        (Value::String(s0), Value::String(s1)) if s0 == s1 => diffs,
        (Value::Object(m0), Value::Object(m1)) => diff_json_map(m0, m1, diffs, path, arr_diff),
        (Value::Array(v0), Value::Array(v1)) => {
            arr_diff.diff(v0.as_slice(), v1.as_slice(), diffs, path)
        }
        (_, _) => {
            // not equal case
            diffs.push(DiffElem {
                diff: DiffChange::Replace {
                    old_val: jval0.clone(),
                    new_val: jval1.clone(),
                },
                path,
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
    arr_diff: &ArrDiffMethod,
) -> Vec<DiffElem> {
    for (k, v0) in m0.iter() {
        if let Some(v1) = m1.get(k) {
            let new_path = path.clone_then_add_key(k);
            diffs = diff_json_inner(v0, v1, diffs, new_path, arr_diff);
        }
    }
    let keys0: HashSet<String> = m0.keys().cloned().collect();
    let keys1: HashSet<String> = m1.keys().cloned().collect();
    let keys_only_in_m0 = keys0.difference(&keys1);
    let keys_only_in_m1 = keys1.difference(&keys0);

    for k in keys_only_in_m0 {
        let new_path = path.clone_then_add_key(k);
        diffs.push(DiffElem {
            diff: DiffChange::Remove(m0.get(k).unwrap().clone()),
            path: new_path,
        })
    }

    for k in keys_only_in_m1 {
        let new_path = path.clone_then_add_key(k);
        diffs.push(DiffElem {
            diff: DiffChange::Add(m1.get(k).unwrap().clone()),
            path: new_path,
        })
    }

    diffs
}

fn diff_json_arr_lcs(
    arr0: &[Value],
    arr1: &[Value],
    mut diffs: Vec<DiffElem>,
    path: Path,
) -> Vec<DiffElem> {
    let mut lcs_pairs = lcs::lcs(arr0, arr1);
    let mut idx0 = 0;
    let mut idx1 = 0;
    let mut shift_idx = 0;

    lcs_pairs.reverse();

    while !lcs_pairs.is_empty() {
        let same_idx_pair = lcs_pairs.last().unwrap();
        assert!(idx0 <= same_idx_pair.0);
        assert!(idx1 <= same_idx_pair.1);
        if idx0 == same_idx_pair.0 && idx1 == same_idx_pair.1 {
            // do nothing
            lcs_pairs.pop();
            shift_idx += 1;
            idx0 += 1;
            idx1 += 1;
        } else if idx0 < same_idx_pair.0 && idx1 < same_idx_pair.1 {
            // replace
            let new_path = path.clone_then_add_idx(shift_idx);
            diffs = diff_json_inner(
                &arr0[idx0],
                &arr1[idx1],
                diffs,
                new_path,
                &ArrDiffMethod::Lcs,
            );
            shift_idx += 1;
            idx0 += 1;
            idx1 += 1;
        } else if idx0 < same_idx_pair.0 && idx1 == same_idx_pair.1 {
            // remove val in arr0
            let new_path = path.clone_then_add_idx(shift_idx);
            diffs.push(DiffElem {
                diff: DiffChange::Remove(arr0[idx0].clone()),
                path: new_path,
            });
            idx0 += 1;
        } else if idx0 == same_idx_pair.0 && idx1 < same_idx_pair.1 {
            // add val in arr1
            let new_path = path.clone_then_add_idx(shift_idx);
            diffs.push(DiffElem {
                diff: DiffChange::Add(arr1[idx1].clone()),
                path: new_path,
            });
            idx1 += 1;
            shift_idx += 1;
        }
    }

    let len0 = arr0.len();
    let len1 = arr1.len();
    while idx0 < len0 && idx1 < len1 {
        // replace
        let new_path = path.clone_then_add_idx(shift_idx);
        diffs = diff_json_inner(
            &arr0[idx0],
            &arr1[idx1],
            diffs,
            new_path,
            &ArrDiffMethod::Lcs,
        );
        shift_idx += 1;
        idx0 += 1;
        idx1 += 1;
    }

    while idx0 < len0 {
        // remove val in arr0
        let new_path = path.clone_then_add_idx(shift_idx);
        diffs.push(DiffElem {
            diff: DiffChange::Remove(arr0[idx0].clone()),
            path: new_path,
        });
        idx0 += 1;
    }

    while idx1 < len1 {
        // add val in arr1
        let new_path = path.clone_then_add_idx(shift_idx);
        diffs.push(DiffElem {
            diff: DiffChange::Add(arr1[idx1].clone()),
            path: new_path,
        });
        idx1 += 1;
        shift_idx += 1;
    }

    diffs
}

#[cfg(test)]
mod tests {
    use crate::diff_json;
    use crate::read_json_str;
    use crate::DiffChange;
    use crate::DiffElem;
    use crate::Path;
    use crate::PathElem;
    use crate::Value;
    use serde_json::Number;

    fn check_diff(original: &str, dest: &str, mut expect_diff: Vec<DiffElem>) {
        let origin_json = read_json_str(original);
        assert!(origin_json.is_ok(), "origin json not valid");
        let dest_json = read_json_str(dest);
        assert!(dest_json.is_ok(), "origin json not valid");
        let mut actual_diff = diff_json(original, dest).unwrap();
        actual_diff.sort_by(|a, b| a.path.partial_cmp(&b.path).unwrap());
        expect_diff.sort_by(|a, b| a.path.partial_cmp(&b.path).unwrap());
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
            path: Path::new(vec![PathElem::Key("x".to_string())]),
        };

        check_diff(s0, s1, vec![diff])
    }

    #[test]
    fn test_same_json() {
        let json = "true";
        check_diff(json, json, vec![]);
    }

    #[test]
    fn test_add_op() {
        let json1 = r#"{"a": 1}"#;
        let json2 = r#"{"a": 1, "new": false}"#;
        let json3 = r#"{"a": 1, "new1": false, "new2": null}"#;
        let json4 = r#"{"a": 1, "b": {"a": true}}"#;
        let json5 = r#"{"a": 1, "b": {"a": true, "b": "new_val"}, "c": null}"#;
        check_diff(
            json1,
            json2,
            vec![DiffElem {
                diff: DiffChange::Add(Value::Bool(false)),
                path: Path::new(vec![PathElem::Key("new".to_string())]),
            }],
        );

        check_diff(
            json1,
            json3,
            vec![
                DiffElem {
                    diff: DiffChange::Add(Value::Bool(false)),
                    path: Path::new(vec![PathElem::Key("new1".to_string())]),
                },
                DiffElem {
                    diff: DiffChange::Add(Value::Null),
                    path: Path::new(vec![PathElem::Key("new2".to_string())]),
                },
            ],
        );

        check_diff(
            json4,
            json5,
            vec![
                DiffElem {
                    diff: DiffChange::Add(Value::String("new_val".to_string())),
                    path: Path::new(vec![
                        PathElem::Key("b".to_string()),
                        PathElem::Key("b".to_string()),
                    ]),
                },
                DiffElem {
                    diff: DiffChange::Add(Value::Null),
                    path: Path::new(vec![PathElem::Key("c".to_string())]),
                },
            ],
        );
    }

    #[test]
    fn test_remove() {
        let json1 = r#"{"a": 1}"#;
        let json2 = r#"{"a": 1, "old": false}"#;
        let json3 = r#"{"a": 1, "old1": false, "old2": null}"#;
        let json4 = r#"{"a": 1, "b": {"a": true}}"#;
        let json5 = r#"{"a": 1, "b": {"a": true, "b": "old_val"}, "c": null}"#;
        check_diff(
            json2,
            json1,
            vec![DiffElem {
                diff: DiffChange::Remove(Value::Bool(false)),
                path: Path::new(vec![PathElem::Key("old".to_string())]),
            }],
        );

        check_diff(
            json3,
            json1,
            vec![
                DiffElem {
                    diff: DiffChange::Remove(Value::Bool(false)),
                    path: Path::new(vec![PathElem::Key("old1".to_string())]),
                },
                DiffElem {
                    diff: DiffChange::Remove(Value::Null),
                    path: Path::new(vec![PathElem::Key("old2".to_string())]),
                },
            ],
        );

        check_diff(
            json5,
            json4,
            vec![
                DiffElem {
                    diff: DiffChange::Remove(Value::String("old_val".to_string())),
                    path: Path::new(vec![
                        PathElem::Key("b".to_string()),
                        PathElem::Key("b".to_string()),
                    ]),
                },
                DiffElem {
                    diff: DiffChange::Remove(Value::Null),
                    path: Path::new(vec![PathElem::Key("c".to_string())]),
                },
            ],
        );
    }

    #[test]
    fn test_arr_diff() {
        let json1 = r#"{"a": [1, 2, 3, 6, 7, 8, 9, 10]}"#;
        let json2 = r#"{"a": [0, 1, 3, 7, 8, 9, 13]}"#;
        check_diff(
            json1,
            json2,
            vec![
                DiffElem {
                    diff: DiffChange::Add(Value::Number(Number::from(0))),
                    path: Path::new(vec![PathElem::Key("a".to_owned()), PathElem::Index(0)]),
                },
                DiffElem {
                    diff: DiffChange::Remove(Value::Number(Number::from(2))),
                    path: Path::new(vec![PathElem::Key("a".to_owned()), PathElem::Index(2)]),
                },
                DiffElem {
                    diff: DiffChange::Remove(Value::Number(Number::from(6))),
                    path: Path::new(vec![PathElem::Key("a".to_owned()), PathElem::Index(3)]),
                },
                DiffElem {
                    diff: DiffChange::Replace {
                        old_val: Value::Number(Number::from(10)),
                        new_val: Value::Number(Number::from(13)),
                    },
                    path: Path::new(vec![PathElem::Key("a".to_owned()), PathElem::Index(6)]),
                },
            ],
        );
    }

    #[test]
    fn test_obj_nested_in_arr() {
        let json1 = r#"
        {
            "key": [
                13,
                {
                    "a": "b",
                    "c": "d"
                },
                10
            ]
        }
        "#;
        let json2 = r#"
        {
            "key": [
                13,
                {
                    "a": "b",
                    "c": "z",
                    "e": "f"
                },
                10
            ]
        }
        "#;
        check_diff(
            json1,
            json2,
            vec![
                DiffElem {
                    diff: DiffChange::Add(Value::String("f".to_owned())),
                    path: Path::new(vec![
                        PathElem::Key("key".to_owned()),
                        PathElem::Index(1),
                        PathElem::Key("e".to_owned()),
                    ]),
                },
                DiffElem {
                    diff: DiffChange::Replace {
                        old_val: Value::String("d".to_owned()),
                        new_val: Value::String("z".to_owned()),
                    },
                    path: Path::new(vec![
                        PathElem::Key("key".to_owned()),
                        PathElem::Index(1),
                        PathElem::Key("c".to_owned()),
                    ]),
                },
            ],
        );

        let json3 = r#"
        {
            "key": [
                13,
                {
                    "a": "b",
                    "c": "d"
                }
            ]
        }
        "#;
        let json4 = r#"
        {
            "key": [
                13,
                {
                    "a": "b",
                    "c": "z",
                    "e": "f"
                }
            ]
        }
        "#;
        check_diff(
            json3,
            json4,
            vec![
                DiffElem {
                    diff: DiffChange::Add(Value::String("f".to_owned())),
                    path: Path::new(vec![
                        PathElem::Key("key".to_owned()),
                        PathElem::Index(1),
                        PathElem::Key("e".to_owned()),
                    ]),
                },
                DiffElem {
                    diff: DiffChange::Replace {
                        old_val: Value::String("d".to_owned()),
                        new_val: Value::String("z".to_owned()),
                    },
                    path: Path::new(vec![
                        PathElem::Key("key".to_owned()),
                        PathElem::Index(1),
                        PathElem::Key("c".to_owned()),
                    ]),
                },
            ],
        );
    }
}
