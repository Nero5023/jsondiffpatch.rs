use crate::{Path, PathElem};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryFrom;
use thiserror::Error;
// TODO: maybe use thiserror Result<T, JsonPatchError>
use anyhow::{anyhow, Result};

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
            Patch::Copy { from } => Operation {
                op: "from".to_string(),
                path: path.to_string(),
                value: None,
                from: Some(from.to_string()),
            },
            Patch::Test(val) => Operation {
                op: "test".to_string(),
                path: path.to_string(),
                value: Some(val),
                from: None,
            },
        }
    }
}

impl TryInto<PatchElem> for Operation {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<PatchElem, Self::Error> {
        let path = Path::try_from(self.path)?;
        match self.op.as_str() {
            "add" => Ok(PatchElem {
                patch: Patch::Add(
                    self.value
                        .ok_or(anyhow!("add operaton does not have 'value' field"))?,
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
                        .ok_or(anyhow!("replace operation does not have 'value' field"))?,
                ),
                path,
            }),
            "move" => Ok(PatchElem {
                patch: Patch::Move {
                    from: self
                        .from
                        .ok_or(anyhow!("move operation does not have 'from' field"))?
                        .try_into()?,
                },
                path,
            }),
            "copy" => Ok(PatchElem {
                patch: Patch::Copy {
                    from: self
                        .from
                        .ok_or(anyhow!("copy operation does not have 'from' field"))?
                        .try_into()?,
                },
                path,
            }),
            "test" => Ok(PatchElem {
                patch: Patch::Test(
                    self.value
                        .ok_or(anyhow!("test operation does not have 'value' filed"))?,
                ),
                path,
            }),
            _ => Err(anyhow!("Unsupport op '{}'", self.op)),
        }
    }
}

#[derive(Debug)]
pub enum Patch {
    Add(Value),
    Remove,
    Replace(Value),
    Move { from: Path },
    Copy { from: Path },
    Test(Value),
}

#[derive(Debug)]
pub struct PatchElem {
    pub patch: Patch,
    pub path: Path,
}

impl TryFrom<&str> for PatchElem {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        let res: std::result::Result<Operation, _> = serde_json::from_str(s);
        match res {
            Ok(op) => Ok(op.try_into()?),
            Err(e) => Err(e.into()),
        }
    }
}

impl TryFrom<String> for PatchElem {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        PatchElem::try_from(value.as_ref())
    }
}

pub struct JsonPatch {
    pub patches: Vec<PatchElem>,
}

impl TryFrom<&str> for JsonPatch {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        let ops_res: std::result::Result<Vec<Operation>, _> = serde_json::from_str(s);
        match ops_res {
            Ok(ops) => {
                let res: std::result::Result<Vec<PatchElem>, _> =
                    ops.into_iter().map(|op| op.try_into()).collect();
                let patches = res?;
                Ok(JsonPatch { patches })
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl TryFrom<String> for JsonPatch {
    type Error = anyhow::Error;

    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        JsonPatch::try_from(s.as_ref())
    }
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
            Patch::Copy { from } => {
                let target = retrieve_json(&json, from)?;
                add_json(&mut clone_json, &mut self.path.clone(), &target)?;
                Ok(clone_json)
            }
            Patch::Test(v) => {
                let target = retrieve_json(&json, &self.path)?;
                if v != target {
                    return Err(JsonPatchError::TestFail {
                        path: self.path.clone(),
                        expected: v.clone(),
                        actual: target.clone(),
                    }
                    .into());
                }
                Ok(clone_json)
            }
        }
    }
}

impl JsonPatch {
    pub fn apply(&self, json: &Value) -> Result<Value> {
        let mut res = json.clone();
        for patch in self.patches.iter() {
            res = patch.apply(&res)?;
        }
        Ok(res)
    }
}

#[derive(Error, Debug)]
pub enum JsonPatchError {
    #[error("Index out of range (index: {index:?}, len: {len:?})")]
    IndexOutOfRange { index: usize, len: usize },

    #[error("Parent node not exit")]
    ParentNodeNotExist,

    #[error("Token is not an array")]
    TokenIsNotAnArray,

    #[error("Token is not an object")]
    TokenIsNotAnObject,

    #[error("Path not exit")]
    PathNotExit,

    #[error(
        "Patch operation `test` fail for path {path:?} (expected {expected:?}, found {actual:?})"
    )]
    TestFail {
        path: Path,
        expected: Value,
        actual: Value,
    },
}

// TODO: add_json use val reference, actually I think it should use ownership
fn add_json(json: &mut Value, path: &mut Path, val: &Value) -> Result<()> {
    if json.is_null() && !path.is_empty() {
        return Err(JsonPatchError::ParentNodeNotExist.into());
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
                    return Err(JsonPatchError::ParentNodeNotExist.into());
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
                    return Err(JsonPatchError::IndexOutOfRange {
                        index: idx,
                        len: arr.len(),
                    }
                    .into());
                }
            } else {
                if idx < arr.len() {
                    return add_json(&mut arr[idx], path, val);
                } else {
                    return Err(JsonPatchError::IndexOutOfRange {
                        index: idx,
                        len: arr.len(),
                    }
                    .into());
                }
            }
        }
        (_, PathElem::Index(_)) => {
            return Err(JsonPatchError::TokenIsNotAnArray.into());
        }
        (_, PathElem::Key(_)) => {
            return Err(JsonPatchError::TokenIsNotAnObject.into());
        }
    }
}

fn remove_json(json: &mut Value, path: &mut Path) -> Result<Value> {
    if json.is_null() && !path.is_empty() {
        return Err(JsonPatchError::ParentNodeNotExist.into());
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
                return Err(JsonPatchError::ParentNodeNotExist.into());
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
                    return Err(JsonPatchError::IndexOutOfRange {
                        index: idx,
                        len: arr.len(),
                    }
                    .into());
                }
            } else {
                if idx < arr.len() {
                    return remove_json(&mut arr[idx], path);
                } else {
                    return Err(JsonPatchError::IndexOutOfRange {
                        index: idx,
                        len: arr.len(),
                    }
                    .into());
                }
            }
        }
        (_, PathElem::Index(_)) => {
            return Err(JsonPatchError::TokenIsNotAnArray.into());
        }
        (_, PathElem::Key(_)) => {
            return Err(JsonPatchError::TokenIsNotAnObject.into());
        }
    }
}

fn replace_json(json: &mut Value, path: &mut Path, val: &Value) -> Result<()> {
    if json.is_null() && !path.is_empty() {
        return Err(JsonPatchError::ParentNodeNotExist.into());
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
                    return Err(JsonPatchError::ParentNodeNotExist.into());
                }
            }
        }
        (Value::Array(arr), PathElem::Index(idx)) => {
            if path.is_empty() {
                if idx < arr.len() {
                    arr[idx] = val.clone();
                    return Ok(());
                } else {
                    return Err(JsonPatchError::IndexOutOfRange {
                        index: idx,
                        len: arr.len(),
                    }
                    .into());
                }
            } else {
                if idx < arr.len() {
                    return replace_json(&mut arr[idx], path, val);
                } else {
                    return Err(JsonPatchError::IndexOutOfRange {
                        index: idx,
                        len: arr.len(),
                    }
                    .into());
                }
            }
        }
        (_, PathElem::Index(_)) => {
            return Err(JsonPatchError::TokenIsNotAnArray.into());
        }
        (_, PathElem::Key(_)) => {
            return Err(JsonPatchError::TokenIsNotAnObject.into());
        }
    }
}

fn retrieve_json<'a>(json: &'a Value, path: &Path) -> Result<&'a Value> {
    let mut current_json = json;
    let path_len = path.len();
    for (idx, path_elem) in path.iter().enumerate() {
        match (current_json, path_elem) {
            (Value::Object(obj), PathElem::Key(key)) => {
                if let Some(child) = current_json.get(key) {
                    if idx == path_len - 1 {
                        return Ok(child);
                    } else {
                        current_json = child;
                    }
                } else {
                    //TODO: check PahtNotExit and ParentNodeNotExist, if need both, and difference
                    return Err(JsonPatchError::PathNotExit.into());
                }
            }
            (Value::Array(arr), PathElem::Index(idx)) => {
                // TODO: move two Err to one if (reorg if)
                let idx = *idx;
                if idx == path_len - 1 {
                    if idx < arr.len() {
                        return Ok(&arr[idx]);
                    } else {
                        return Err(JsonPatchError::IndexOutOfRange {
                            index: idx,
                            len: arr.len(),
                        }
                        .into());
                    }
                } else {
                    if idx < arr.len() {
                        current_json = &arr[idx];
                    } else {
                        return Err(JsonPatchError::IndexOutOfRange {
                            index: idx,
                            len: arr.len(),
                        }
                        .into());
                    }
                }
            }
            (_, PathElem::Index(_)) => {
                return Err(JsonPatchError::TokenIsNotAnArray.into());
            }
            (_, PathElem::Key(_)) => {
                return Err(JsonPatchError::TokenIsNotAnObject.into());
            }
        }
    }
    unreachable!()
}

