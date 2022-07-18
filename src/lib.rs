use core::result;
use serde_json::map::Map;
use serde_json::{Result, Value};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path;

pub fn read_json_str(s: &str) -> Result<Value> {
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
pub enum PathElem {
    Key(String),
    Index(usize),
}

type Path = Vec<PathElem>;

// TODO: use reference not own
#[derive(Debug, PartialEq, Eq)]
pub struct DiffElem {
    diff: DiffChange,
    path: Path,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DiffChange {
    Replace { old_val: Value, new_val: Value },
    Add(Value),
    Remove(Value),
}

enum JsonArrDiff {
    Simple,
    Lcs,
}

impl JsonArrDiff {
    fn diff(
        &self,
        arr0: &[Value],
        arr1: &[Value],
        mut diffs: Vec<DiffElem>,
        path: Path,
    ) -> Vec<DiffElem> {
        match *self {
            JsonArrDiff::Simple => {
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
            JsonArrDiff::Lcs => diff_json_arr_lcs(arr0, arr1, diffs, path),
        }
    }
}

pub fn diff_json(jval0: &Value, jval1: &Value, diffs: Vec<DiffElem>, path: Path) -> Vec<DiffElem> {
    diff_json_inner(jval0, jval1, diffs, path, &JsonArrDiff::Lcs)
}

pub fn diff_json_simple(
    jval0: &Value,
    jval1: &Value,
    diffs: Vec<DiffElem>,
    path: Path,
) -> Vec<DiffElem> {
    diff_json_inner(jval0, jval1, diffs, path, &JsonArrDiff::Simple)
}

fn diff_json_inner(
    jval0: &Value,
    jval1: &Value,
    mut diffs: Vec<DiffElem>,
    path: Path,
    arr_diff: &JsonArrDiff,
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
    arr_diff: &JsonArrDiff,
) -> Vec<DiffElem> {
    for (k, v0) in m0.iter() {
        if let Some(v1) = m1.get(k) {
            let mut new_path = path.clone();
            new_path.push(PathElem::Key(k.to_string()));
            diffs = diff_json_inner(v0, v1, diffs, new_path, arr_diff);
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

fn diff_json_arr_lcs(
    arr0: &[Value],
    arr1: &[Value],
    mut diffs: Vec<DiffElem>,
    path: Path,
) -> Vec<DiffElem> {
    let mut lcs_pairs = lcs(arr0, arr1);
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
            let mut new_path = path.clone();
            new_path.push(PathElem::Index(shift_idx));
            diffs.push(DiffElem {
                diff: DiffChange::Replace {
                    old_val: arr0[idx0].clone(),
                    new_val: arr1[idx1].clone(),
                },
                path: new_path,
            });
            shift_idx += 1;
            idx0 += 1;
            idx1 += 1;
        } else if idx0 < same_idx_pair.0 && idx1 == same_idx_pair.1 {
            // remove val in arr0
            let mut new_path = path.clone();
            new_path.push(PathElem::Index(shift_idx));
            diffs.push(DiffElem {
                diff: DiffChange::Remove(arr0[idx0].clone()),
                path: new_path,
            });
            idx0 += 1;
        } else if idx0 == same_idx_pair.0 && idx1 < same_idx_pair.1 {
            // add val in arr1
            let mut new_path = path.clone();
            new_path.push(PathElem::Index(shift_idx));
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
        let mut new_path = path.clone();
        new_path.push(PathElem::Index(shift_idx));
        diffs.push(DiffElem {
            diff: DiffChange::Replace {
                old_val: arr0[idx0].clone(),
                new_val: arr1[idx1].clone(),
            },
            path: new_path,
        });
        shift_idx += 1;
        idx0 += 1;
        idx1 += 1;
    }

    while idx0 < len0 {
        // remove val in arr0
        let mut new_path = path.clone();
        new_path.push(PathElem::Index(shift_idx));
        diffs.push(DiffElem {
            diff: DiffChange::Remove(arr0[idx0].clone()),
            path: new_path,
        });
        idx0 += 1;
    }

    while idx1 < len1 {
        // add val in arr1
        let mut new_path = path.clone();
        new_path.push(PathElem::Index(shift_idx));
        diffs.push(DiffElem {
            diff: DiffChange::Add(arr1[idx1].clone()),
            path: new_path,
        });
        idx1 += 1;
        shift_idx += 1;
    }

    diffs
}

fn lcs<T: PartialEq>(arr0: &[T], arr1: &[T]) -> Vec<(usize, usize)> {
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

    fn str2vec(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn test_lcs_empty_and_another() {
        let lcs1 = lcs(&str2vec(""), &str2vec("abcdef"));
        assert_eq!(lcs1.len(), 0);
        let lcs2 = lcs(&str2vec("abcdef"), &str2vec(""));
        assert_eq!(lcs2.len(), 0);
    }

    #[test]
    fn test_lcs_two_same_seq() {
        let s = "abcde";
        let lcs_vec = lcs(&str2vec(s), &str2vec(s));
        assert_eq!(lcs_vec, vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)]);
    }

    #[test]
    fn test_lcs_prefix_seq() {
        let s = "abcdef";
        let prefix = "abc";
        let lcs1 = lcs(&str2vec(s), &str2vec(prefix));
        assert_eq!(lcs1, vec![(0, 0), (1, 1), (2, 2)]);

        let lcs2 = lcs(&str2vec(prefix), &str2vec(s));
        assert_eq!(lcs2, vec![(0, 0), (1, 1), (2, 2)]);
    }

    #[test]
    fn test_lcs_no_common_seq() {
        let lcs_vec = lcs(&str2vec("abcdef"), &str2vec("ghijkl"));
        assert_eq!(lcs_vec.len(), 0);
    }

    #[test]
    fn test_lcs_same_in_mid() {
        let s1 = "abcdefgh";
        let s2 = "bdeg";
        let lcs_vec = lcs(&str2vec(s1), &str2vec(s2));
        assert_eq!(lcs_vec, vec![(1, 0), (3, 1), (4, 2), (6, 3)]);
    }

    #[test]
    fn test_lcs_repeat_elem() {
        let s1 = "abcbdbebf";
        let s2 = "bbbb";
        let lcs_vec = lcs(&str2vec(s1), &str2vec(s2));
        assert_eq!(lcs_vec, vec![(1, 0), (3, 1), (5, 2), (7, 3)]);
    }

    #[test]
    fn test_lcs_non_unique_elem() {
        let s1 = "abcdabcd";
        let s2 = "gabhakbf";
        let lcs_vec = lcs(&str2vec(s1), &str2vec(s2));
        assert_eq!(lcs_vec, vec![(0, 1), (1, 2), (4, 4), (5, 6)]);
    }

    #[test]
    fn test_lcs_common_pre_suf() {
        let s1 = "abctotodef";
        let s2 = "abctatatadef";
        let lcs_vec = lcs(&str2vec(s1), &str2vec(s2));
        assert_eq!(
            lcs_vec,
            vec![
                (0, 0),
                (1, 1),
                (2, 2),
                (3, 3),
                (5, 5),
                (7, 9),
                (8, 10),
                (9, 11)
            ]
        );
    }
}
