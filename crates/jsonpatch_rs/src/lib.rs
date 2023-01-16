// use crate::{Path, PathElem};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryFrom;
use thiserror::Error;
// TODO: maybe use thiserror Result<T, JsonPatchError>
use anyhow::{anyhow, Result};

use jsonptr::operate_by_jsonptr::*;
use jsonptr::pointer::JsonPointer;

#[derive(Serialize, Deserialize)]
struct Operation {
    op: String,
    path: String,
    value: Option<Value>,
    from: Option<String>,
}

impl From<PatchElem> for Operation {
    fn from(patch_elem: PatchElem) -> Self {
        let ptr = patch_elem.json_ptr;
        match patch_elem.patch {
            Patch::Add(val) => Operation {
                op: "add".to_owned(),
                path: ptr.to_escaped_string(),
                value: Some(val),
                from: None,
            },
            Patch::Remove => Operation {
                op: "remove".to_owned(),
                path: ptr.to_escaped_string(),
                value: None,
                from: None,
            },
            Patch::Replace(val) => Operation {
                op: "move".to_string(),
                path: ptr.to_escaped_string(),
                value: Some(val),
                from: None,
            },
            Patch::Move { from } => Operation {
                op: "from".to_string(),
                path: ptr.to_escaped_string(),
                value: None,
                from: Some(from.to_escaped_string()),
            },
            Patch::Copy { from } => Operation {
                op: "from".to_string(),
                path: ptr.to_escaped_string(),
                value: None,
                from: Some(from.to_escaped_string()),
            },
            Patch::Test(val) => Operation {
                op: "test".to_string(),
                path: ptr.to_escaped_string(),
                value: Some(val),
                from: None,
            },
        }
    }
}

impl TryInto<PatchElem> for Operation {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<PatchElem, Self::Error> {
        let json_ptr = JsonPointer::try_from(self.path)?;
        match self.op.as_str() {
            "add" => Ok(PatchElem {
                patch: Patch::Add(
                    self.value
                        .ok_or(anyhow!("add operaton does not have 'value' field"))?,
                ),
                json_ptr,
            }),
            "remove" => Ok(PatchElem {
                patch: Patch::Remove,
                json_ptr,
            }),
            "replace" => Ok(PatchElem {
                patch: Patch::Replace(
                    self.value
                        .ok_or(anyhow!("replace operation does not have 'value' field"))?,
                ),
                json_ptr,
            }),
            "move" => Ok(PatchElem {
                patch: Patch::Move {
                    from: self
                        .from
                        .ok_or(anyhow!("move operation does not have 'from' field"))?
                        .try_into()?,
                },
                json_ptr,
            }),
            "copy" => Ok(PatchElem {
                patch: Patch::Copy {
                    from: self
                        .from
                        .ok_or(anyhow!("copy operation does not have 'from' field"))?
                        .try_into()?,
                },
                json_ptr,
            }),
            "test" => Ok(PatchElem {
                patch: Patch::Test(
                    self.value
                        .ok_or(anyhow!("test operation does not have 'value' filed"))?,
                ),
                json_ptr,
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
    Move { from: JsonPointer },
    Copy { from: JsonPointer },
    Test(Value),
}

#[derive(Debug)]
pub struct PatchElem {
    patch: Patch,
    json_ptr: JsonPointer,
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
            Patch::Add(v) => {
                clone_json.add(&self.json_ptr, v.clone())?;
                Ok(clone_json)
            }
            Patch::Remove => {
                clone_json.delete(&self.json_ptr)?;
                Ok(clone_json)
            }
            Patch::Replace(val) => {
                clone_json.replace(&self.json_ptr, val.clone())?;
                Ok(clone_json)
            }
            Patch::Move { from } => {
                let removed_val = clone_json.delete(from)?;
                clone_json.add(&self.json_ptr, removed_val)?;
                Ok(clone_json)
            }
            Patch::Copy { from } => {
                let target = json.get_by_ptr(from)?;
                clone_json.add(&self.json_ptr, target.clone())?;
                Ok(clone_json)
            }
            Patch::Test(v) => {
                let target = json.get_by_ptr(&self.json_ptr)?;
                if v != target {
                    return Err(JsonPatchError::TestFail {
                        json_ptr: self.json_ptr.clone(),
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
        "Patch operation `test` fail for path {json_ptr:?} (expected {expected:?}, found {actual:?})"
    )]
    TestFail {
        json_ptr: JsonPointer,
        expected: Value,
        actual: Value,
    },
}
