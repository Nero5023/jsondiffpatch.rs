use crate::Path;
use crate::PathElem;
use core::result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize)]
struct Operation {
    op: String,
    path: String,
    value: Option<Value>,
    from: Option<String>,
}

impl From<PatchElem> for Operation {
    fn from(patch_elem: PatchElem) -> Self {
        let path = patch_elem.path;
        match patch_elem.patch {
            Patch::Add(val) => Operation {
                op: "add".to_owned(),
                path: path.to_string(),
                value: Some(val),
                from: None,
            },
            Patch::Remove => Operation {
                op: "remove".to_owned(),
                path: path.to_string(),
                value: None,
                from: None,
            },
            Patch::Replace(val) => Operation {
                op: "move".to_string(),
                path: path.to_string(),
                value: Some(val),
                from: None,
            },
            Patch::Move { from } => Operation {
                op: "from".to_string(),
                path: path.to_string(),
                value: None,
                from: Some(from.to_string()),
            },
        }
    }
}

impl TryInto<PatchElem> for Operation {
    type Error = String;

    fn try_into(self) -> std::result::Result<PatchElem, Self::Error> {
        let path = Path::try_from(self.path)?;
        match self.op.as_str() {
            "add" => Ok(PatchElem {
                patch: Patch::Add(
                    self.value
                        .ok_or("add operaton does not have 'value' field")?,
                ),
                path,
            }),
            "remove" => Ok(PatchElem {
                patch: Patch::Remove,
                path,
            }),
            "replace" => Ok(PatchElem {
                patch: Patch::Replace(
                    self.value
                        .ok_or("replace operation does not have 'value' field")?,
                ),
                path,
            }),
            "move" => Ok(PatchElem {
                patch: Patch::Move {
                    from: self
                        .from
                        .ok_or("move operation does not have 'from' field")?
                        .try_into()?,
                },
                path,
            }),
            _ => Err(format!("Unsupport op '{}'", self.op)),
        }
    }
}

#[derive(Debug)]
pub enum Patch {
    Add(Value),
    Remove,
    Replace(Value),
    Move { from: Path },
}

#[derive(Debug)]
pub struct PatchElem {
    patch: Patch,
    path: Path,
}

impl TryFrom<&str> for PatchElem {
    type Error = String;

    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        let res: std::result::Result<Operation, _> = serde_json::from_str(s);
        match res {
            Ok(op) => Ok(op.try_into()?),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub struct JsonPatch {
    patches: Vec<PatchElem>,
}

impl PatchElem {
    fn apply(&self, json: &Value) -> Result<Value> {
        let mut clone_json = json.clone();
        match &self.patch {
            // TODO: refactor code
            Patch::Add(v) => {
                add_json(&mut clone_json, &mut self.path.clone(), &v)?;
                Ok(clone_json)
            }
            Patch::Remove => {
                remove_json(&mut clone_json, &mut self.path.clone())?;
                Ok(clone_json)
            }
            Patch::Replace(val) => {
                replace_json(&mut clone_json, &mut self.path.clone(), &val)?;
                Ok(clone_json)
            }
            Patch::Move { from } => {
                let removed_val = remove_json(&mut clone_json, &mut from.clone())?;
                add_json(&mut clone_json, &mut self.path.clone(), &removed_val)?;
                Ok(clone_json)
            }
            _ => todo!(),
        }
    }
}

impl JsonPatch {
    fn apply(&self, json: &Value) -> Result<Value> {
        let mut res = json.clone();
        for patch in self.patches.iter() {
            res = patch.apply(&res)?;
        }
        Ok(res)
    }
}

#[derive(Debug)]
pub struct Error {
    err: Box<ErrorCode>,
}

#[derive(Debug)]
pub(crate) enum ErrorCode {
    IndexOutOfRange { index: usize, len: usize },
    ParentNodeNotExist,
    TokenIsNotAnArray,
    TokenIsNotAnObject,
    PathNotExist,
}

pub type Result<T> = result::Result<T, Error>;

fn add_json(json: &mut Value, path: &mut Path, val: &Value) -> Result<()> {
    if json.is_null() && !path.is_empty() {
        return Err(Error {
            err: Box::new(ErrorCode::ParentNodeNotExist),
        });
    }
    if path.is_empty() {
        *json = val.clone();
        return Ok(());
    }
    let path_elem = path.remove(0);

    match (json, path_elem) {
        (Value::Object(obj), PathElem::Key(key)) => {
            if obj.contains_key(&key) {
                return add_json(&mut obj[&key], path, val);
            } else {
                if path.is_empty() {
                    obj.insert(key.clone(), val.clone());
                    return Ok(());
                } else {
                    return Err(Error {
                        err: Box::new(ErrorCode::ParentNodeNotExist),
                    });
                }
            }
            //return add_json(&mut json[&key], path, val);
        }
        (Value::Array(arr), PathElem::Index(idx)) => {
            if path.is_empty() {
                // last PathElem is index
                if idx <= arr.len() {
                    arr.insert(idx, val.clone());
                    return Ok(());
                } else {
                    return Err(Error {
                        err: Box::new(ErrorCode::IndexOutOfRange {
                            index: idx,
                            len: arr.len(),
                        }),
                    });
                }
            } else {
                if idx < arr.len() {
                    return add_json(&mut arr[idx], path, val);
                } else {
                    return Err(Error {
                        err: Box::new(ErrorCode::IndexOutOfRange {
                            index: idx,
                            len: arr.len(),
                        }),
                    });
                }
            }
        }
        (_, PathElem::Index(_)) => {
            return Err(Error {
                err: Box::new(ErrorCode::TokenIsNotAnArray),
            });
        }
        (_, PathElem::Key(_)) => {
            return Err(Error {
                err: Box::new(ErrorCode::TokenIsNotAnObject),
            });
        }
    }
}

fn remove_json(json: &mut Value, path: &mut Path) -> Result<Value> {
    if json.is_null() && !path.is_empty() {
        return Err(Error {
            err: Box::new(ErrorCode::ParentNodeNotExist),
        });
    }

    // TODO: this case is for init state, json is some Value, but path is empty
    if path.is_empty() {
        return Ok(json.clone());
    }
    let path_elem = path.remove(0);

    match (json, path_elem) {
        (Value::Object(obj), PathElem::Key(key)) => {
            if let Some(child) = obj.get(&key) {
                if path.is_empty() {
                    let removed_val = child.clone();
                    obj.remove(&key);
                    return Ok(removed_val);
                } else {
                    return remove_json(&mut obj[&key], path);
                }
            } else {
                return Err(Error {
                    err: Box::new(ErrorCode::PathNotExist),
                });
            }
        }
        (Value::Array(arr), PathElem::Index(idx)) => {
            // TODO: move two Err to one if (reorg if)
            if path.is_empty() {
                if idx < arr.len() {
                    let remove_val = arr[idx].clone();
                    arr.remove(idx);
                    return Ok(remove_val);
                } else {
                    return Err(Error {
                        err: Box::new(ErrorCode::IndexOutOfRange {
                            index: idx,
                            len: arr.len(),
                        }),
                    });
                }
            } else {
                if idx < arr.len() {
                    return remove_json(&mut arr[idx], path);
                } else {
                    return Err(Error {
                        err: Box::new(ErrorCode::IndexOutOfRange {
                            index: idx,
                            len: arr.len(),
                        }),
                    });
                }
            }
        }
        (_, PathElem::Index(_)) => {
            return Err(Error {
                err: Box::new(ErrorCode::TokenIsNotAnArray),
            });
        }
        (_, PathElem::Key(_)) => {
            return Err(Error {
                err: Box::new(ErrorCode::TokenIsNotAnObject),
            });
        }
    }
}

fn replace_json(json: &mut Value, path: &mut Path, val: &Value) -> Result<()> {
    if json.is_null() && !path.is_empty() {
        return Err(Error {
            err: Box::new(ErrorCode::ParentNodeNotExist),
        });
    }

    if path.is_empty() {
        *json = val.clone();
        return Ok(());
    }
    let path_elem = path.remove(0);

    match (json, path_elem) {
        (Value::Object(obj), PathElem::Key(key)) => {
            if obj.contains_key(&key) {
                return replace_json(&mut obj[&key], path, val);
            } else {
                if path.is_empty() {
                    obj[&key] = val.clone();
                    return Ok(());
                } else {
                    return Err(Error {
                        err: Box::new(ErrorCode::ParentNodeNotExist),
                    });
                }
            }
        }
        (Value::Array(arr), PathElem::Index(idx)) => {
            if path.is_empty() {
                if idx < arr.len() {
                    arr[idx] = val.clone();
                    return Ok(());
                } else {
                    return Err(Error {
                        err: Box::new(ErrorCode::IndexOutOfRange {
                            index: idx,
                            len: arr.len(),
                        }),
                    });
                }
            } else {
                if idx < arr.len() {
                    return replace_json(&mut arr[idx], path, val);
                } else {
                    return Err(Error {
                        err: Box::new(ErrorCode::IndexOutOfRange {
                            index: idx,
                            len: arr.len(),
                        }),
                    });
                }
            }
        }
        (_, PathElem::Index(_)) => {
            return Err(Error {
                err: Box::new(ErrorCode::TokenIsNotAnArray),
            });
        }
        (_, PathElem::Key(_)) => {
            return Err(Error {
                err: Box::new(ErrorCode::TokenIsNotAnObject),
            });
        }
    }
}

#[cfg(test)]
mod tests {

    use super::JsonPatch;
    use super::Patch;
    use super::PatchElem;
    use super::Path;
    use super::PathElem;
    use serde_json::json;
    use serde_json::{Result, Value};

    #[test]
    fn add_simple_key() -> Result<()> {
        let data = r#"
        {
            "foo": {
                "bar": 2
            }
        }
        "#;
        let patch = PatchElem {
            patch: Patch::Add(json!("hello")),
            path: Path::new(vec![
                PathElem::Key("foo".to_string()),
                PathElem::Key("baz".to_string()),
            ]),
        };

        let jp = JsonPatch {
            patches: vec![patch],
        };
        let res = jp.apply(&serde_json::from_str(data)?).unwrap();
        let expected_str = r#"
        {
            "foo": {
                "bar": 2,
                "baz": "hello"
            }
        }
        "#;
        let expected: Value = serde_json::from_str(expected_str)?;
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn add_simple_index_key() -> Result<()> {
        let data = r#"
        {
            "foo": {
                "bar": 2,
                "baz": [1, 3]
            }
        }
        "#;
        let patch = PatchElem {
            patch: Patch::Add(json!(2)),
            path: Path::new(vec![
                PathElem::Key("foo".to_string()),
                PathElem::Key("baz".to_string()),
                PathElem::Index(1),
            ]),
        };

        let jp = JsonPatch {
            patches: vec![patch],
        };
        let res = jp.apply(&serde_json::from_str(data)?).unwrap();
        let expected_str = r#"
        {
            "foo": {
                "bar": 2,
                "baz": [1, 2, 3]
            }
        }
        "#;
        let expected: Value = serde_json::from_str(expected_str)?;
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn remove_simple_key() -> Result<()> {
        let data = r#"
        {
            "foo": {
                "bar": 2,
                "baz": "hello"
            }
        }
        "#;
        let patch = PatchElem {
            patch: Patch::Remove,
            path: Path::new(vec![
                PathElem::Key("foo".to_string()),
                PathElem::Key("baz".to_string()),
            ]),
        };

        let jp = JsonPatch {
            patches: vec![patch],
        };
        let res = jp.apply(&serde_json::from_str(data)?).unwrap();
        let expected_str = r#"
        {
            "foo": {
                "bar": 2
            }
        }
        "#;
        let expected: Value = serde_json::from_str(expected_str)?;
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn remove_simple_index_key() -> Result<()> {
        let data = r#"
        {
            "foo": {
                "bar": 2,
                "baz": [1, 2, 3]
            }
        }
        "#;
        let patch = PatchElem {
            patch: Patch::Remove,
            path: Path::new(vec![
                PathElem::Key("foo".to_string()),
                PathElem::Key("baz".to_string()),
                PathElem::Index(1),
            ]),
        };

        let jp = JsonPatch {
            patches: vec![patch],
        };
        let res = jp.apply(&serde_json::from_str(data)?).unwrap();
        let expected_str = r#"
        {
            "foo": {
                "bar": 2,
                "baz": [1, 3]
            }
        }
        "#;
        let expected: Value = serde_json::from_str(expected_str)?;
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn replace_simple_key() -> Result<()> {
        let data = r#"
        {
            "foo": {
                "bar": 2,
                "baz": "world"
            }
        }
        "#;
        let patch = PatchElem {
            patch: Patch::Replace(json!("hello")),
            path: Path::new(vec![
                PathElem::Key("foo".to_string()),
                PathElem::Key("baz".to_string()),
            ]),
        };

        let jp = JsonPatch {
            patches: vec![patch],
        };
        let res = jp.apply(&serde_json::from_str(data)?).unwrap();
        let expected_str = r#"
        {
            "foo": {
                "bar": 2,
                "baz": "hello"
            }
        }
        "#;
        let expected: Value = serde_json::from_str(expected_str)?;
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn replace_simple_index_key() -> Result<()> {
        let data = r#"
        {
            "foo": {
                "bar": 2,
                "baz": [1, 2, 3]
            }
        }
        "#;
        let patch = PatchElem {
            patch: Patch::Replace(json!("hello")),
            path: Path::new(vec![
                PathElem::Key("foo".to_string()),
                PathElem::Key("baz".to_string()),
                PathElem::Index(1),
            ]),
        };

        let jp = JsonPatch {
            patches: vec![patch],
        };
        let res = jp.apply(&serde_json::from_str(data)?).unwrap();
        let expected_str = r#"
        {
            "foo": {
                "bar": 2,
                "baz": [1, "hello", 3]
            }
        }
        "#;
        let expected: Value = serde_json::from_str(expected_str)?;
        assert_eq!(res, expected);
        Ok(())
    }
}
