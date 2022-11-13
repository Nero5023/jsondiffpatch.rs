use crate::Path;
use crate::PathElem;
use core::result;
use serde_json::Value;

#[derive(Debug)]
pub enum Patch {
    Add(Value),
    Remove,
    Replace(Value),
}

#[derive(Debug)]
pub struct PatchElem {
    patch: Patch,
    path: Path,
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
                // TODO: check if need add check path empty here
                if self.path.is_empty() {
                    Ok(v.clone())
                } else {
                    add_json(&mut clone_json, &mut self.path.clone(), &v)?;
                    Ok(clone_json)
                }
            }
            Patch::Remove => {
                remove_json(&mut clone_json, &mut self.path.clone())?;
                Ok(clone_json)
            }
            Patch::Replace(val) => {
                replace_json(&mut clone_json, &mut self.path.clone(), &val)?;
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
    println!("{}", json);
    println!("{}", json.is_object());
    println!("{}", json.is_array());
    println!("{}", json.is_string());

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

fn remove_json(json: &mut Value, path: &mut Path) -> Result<()> {
    if json.is_null() && !path.is_empty() {
        return Err(Error {
            err: Box::new(ErrorCode::ParentNodeNotExist),
        });
    }
    // if path.is_empty() {
    //     *json = val.clone();
    //     return Ok(());
    // }
    if path.is_empty() {
        return Ok(());
    }
    let path_elem = path.remove(0);

    match (json, path_elem) {
        (Value::Object(obj), PathElem::Key(key)) => {
            if obj.contains_key(&key) {
                return remove_json(&mut obj[&key], path);
            } else {
                if path.is_empty() {
                    obj.remove(&key);
                    return Ok(());
                } else {
                    return Err(Error {
                        err: Box::new(ErrorCode::PathNotExist),
                    });
                }
            }
        }
        (Value::Array(arr), PathElem::Index(idx)) => {
            // TODO: move two Err to one if (reorg if)
            if path.is_empty() {
                if idx < arr.len() {
                    arr.remove(idx);
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
        _ => todo!(),
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
}
