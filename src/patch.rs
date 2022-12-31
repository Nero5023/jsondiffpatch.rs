// use crate::{Path, PathElem};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryFrom;
use thiserror::Error;
// TODO: maybe use thiserror Result<T, JsonPatchError>
use anyhow::{anyhow, Result};

use crate::jsonptr::operate_by_jsonptr::*;
use crate::jsonptr::pointer::JsonPointer;

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
    patches: Vec<PatchElem>,
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
    // TODO: maybe json need use &mut Value type
    fn apply(&self, json: &Value) -> Result<Value> {
        let mut clone_json = json.clone();
        match &self.patch {
            // TODO: refactor code
            Patch::Add(v) => {
                // add_json(&mut clone_json, &mut self.json_ptr.clone(), &v)?;
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
                // let target = retrieve_json(&json, &self.path)?;
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
    fn apply(&self, json: &Value) -> Result<Value> {
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

#[cfg(test)]
mod tests {

    use super::JsonPatch;
    use super::JsonPatchError;
    use super::PatchElem;
    use anyhow::anyhow;
    use anyhow::Result;
    use serde_json::Value;

    #[test]
    fn add_simple_key() -> Result<()> {
        let data = r#"
        {
            "foo": {
                "bar": 2
            }
        }
        "#;
        let patches_str = r#"
            [
                { "op": "add", "path": "/foo/baz", "value": "hello" }
            ]
            "#;

        let expected_str = r#"
        {
            "foo": {
                "bar": 2,
                "baz": "hello"
            }
        }
        "#;
        test_json_patch_arr(data, patches_str, expected_str)?;
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
        let patches_str = r#"
            [
                { "op": "add", "path": "/foo/baz/1", "value": 2 }
            ]
            "#;

        let expected_str = r#"
        {
            "foo": {
                "bar": 2,
                "baz": [1, 2, 3]
            }
        }
        "#;
        test_json_patch_arr(data, patches_str, expected_str)?;
        Ok(())
    }

    #[test]
    fn add_an_array_element() -> Result<()> {
        let data = r#"{ "foo": [ "bar", "baz" ] }"#;
        let patches_str = r#"
            [
                { "op": "add", "path": "/foo/1", "value": "qux" }
            ]
            "#;
        let expected_str = r#"{ "foo": [ "bar", "qux", "baz" ] }"#;
        test_json_patch_arr(data, patches_str, expected_str)?;
        Ok(())
    }

    #[test]
    fn add_an_object_member() -> Result<()> {
        let data = r#"{ "foo": "bar"}"#;
        let patches_str = r#"
            [
                { "op": "add", "path": "/baz", "value": "qux" }
            ]"#;
        let expected_str = r#"{
                "baz": "qux",
                "foo": "bar"
            }"#;
        test_json_patch_arr(data, patches_str, expected_str)?;
        Ok(())
    }

    #[test]
    fn add_a_nested_member_object() -> Result<()> {
        let data = r#"{ "foo": "bar"}"#;
        let patches_str = r#"
            [
                { "op": "add", "path": "/child", "value": { "grandchild": { } } }
            ]
            "#;
        let expected_str = r#"
                {
                    "foo": "bar",
                    "child": {
                        "grandchild": {}
                    }
                }
                "#;
        test_json_patch_arr(data, patches_str, expected_str)?;
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
        let patches_str = r#"
            [
                { "op": "remove", "path": "/foo/baz"}
            ]
            "#;

        let expected_str = r#"
        {
            "foo": {
                "bar": 2
            }
        }
        "#;
        test_json_patch_arr(data, patches_str, expected_str)?;
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
        let patches_str = r#"
            [
                { "op": "remove", "path": "/foo/baz/1"}
            ]
            "#;

        let expected_str = r#"
        {
            "foo": {
                "bar": 2,
                "baz": [1, 3]
            }
        }
        "#;
        test_json_patch_arr(data, patches_str, expected_str)?;
        Ok(())
    }

    #[test]
    fn remove_an_object_member() -> Result<()> {
        let data = r#"{
                "baz": "qux",
                "foo": "bar"
            }"#;
        let patches_str = r#"
            [
                { "op": "remove", "path": "/baz" }
            ]
            "#;
        let expected_str = r#"{ "foo": "bar" }"#;
        test_json_patch_arr(data, patches_str, expected_str)?;
        Ok(())
    }

    #[test]
    fn remove_an_array_element() -> Result<()> {
        let data = r#"{ "foo": [ "bar", "qux", "baz" ] }"#;
        let patches_str = r#"
            [
                { "op": "remove", "path": "/foo/1" }
            ]
            "#;
        let expected_str = r#"{ "foo": [ "bar", "baz" ] }"#;
        test_json_patch_arr(data, patches_str, expected_str)?;
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
        let patches_str = r#"
            [
                { "op": "replace", "path": "/foo/baz", "value": "hello" }
            ]
            "#;

        let expected_str = r#"
        {
            "foo": {
                "bar": 2,
                "baz": "hello"
            }
        }
        "#;
        test_json_patch_arr(data, patches_str, expected_str)?;
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
        let patches_str = r#"
            [
                { "op": "replace", "path": "/foo/baz/1", "value": "hello" }
            ]
            "#;

        let expected_str = r#"
        {
            "foo": {
                "bar": 2,
                "baz": [1, "hello", 3]
            }
        }
        "#;
        test_json_patch_arr(data, patches_str, expected_str)?;
        Ok(())
    }

    #[test]
    fn replace_a_value() -> Result<()> {
        let data = r#"
            {
                "baz": "qux",
                "foo": "bar"
            }"#;
        let patches_str = r#"
            [
                { "op": "replace", "path": "/baz", "value": "boo" }
            ]
            "#;
        let expected_str = r#"
                {
                    "baz": "boo",
                    "foo": "bar"
                }
                "#;
        test_json_patch_arr(data, patches_str, expected_str)?;
        Ok(())
    }

    fn test_json_patch(json: &str, patch_str: &str, expected_json_str: &str) -> Result<()> {
        let patch: PatchElem = PatchElem::try_from(patch_str)?;
        let jp = JsonPatch {
            patches: vec![patch],
        };
        let res = jp.apply(&serde_json::from_str(json)?)?;
        let expected: Value = serde_json::from_str(expected_json_str)?;
        assert_eq!(res, expected);
        Ok(())
    }

    fn test_json_patch_arr(json: &str, patches_str: &str, expected_json_str: &str) -> Result<()> {
        let jp: JsonPatch = JsonPatch::try_from(patches_str)?;
        let res = jp.apply(&serde_json::from_str(json)?)?;
        let expected: Value = serde_json::from_str(expected_json_str)?;
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn move_a_value() -> Result<()> {
        let data = r#"
            {
                "foo": {
                    "bar": "baz",
                    "waldo": "fred"
            },
                "qux": {
                    "corge": "grault"
                }
            }"#;
        let patch_str = r#"{ "op": "move", "from": "/foo/waldo", "path": "/qux/thud" }"#;
        let expected_str = r#"
            {
                "foo": {
                    "bar": "baz"
                },
                "qux": {
                    "corge": "grault",
                    "thud": "fred"
                }
           }"#;

        test_json_patch(data, patch_str, expected_str)?;
        Ok(())
    }

    #[test]
    fn move_an_array_element() -> Result<()> {
        let data = r#"{ "foo": [ "all", "grass", "cows", "eat" ] }"#;
        let patch_str = r#"{ "op": "move", "from": "/foo/1", "path": "/foo/3" }"#;
        let expected_str = r#"{ "foo": [ "all", "cows", "eat", "grass" ] }"#;
        test_json_patch(data, patch_str, expected_str)?;
        Ok(())
    }

    #[test]
    fn copy_a_value() -> Result<()> {
        let data = r#"
            {
                "foo": {
                    "bar": "baz",
                    "waldo": "fred"
            },
                "qux": {
                    "corge": "grault"
                }
            }"#;
        let patch_str = r#"{ "op": "copy", "from": "/foo/waldo", "path": "/qux/thud" }"#;
        let expected_str = r#"
            {
                "foo": {
                    "bar": "baz",
                    "waldo": "fred"
                },
                "qux": {
                    "corge": "grault",
                    "thud": "fred"
                }
           }"#;

        test_json_patch(data, patch_str, expected_str)?;
        Ok(())
    }

    #[test]
    fn copy_an_array_element() -> Result<()> {
        let data = r#"{ "foo": [ "all", "grass", "cows", "eat" ] }"#;
        let patch_str = r#"{ "op": "copy", "from": "/foo/1", "path": "/foo/3" }"#;
        let expected_str = r#"{ "foo": [ "all", "grass", "cows", "grass", "eat" ] }"#;
        test_json_patch(data, patch_str, expected_str)?;
        Ok(())
    }

    #[test]
    fn test_a_value_success() -> Result<()> {
        let data = r#"
           {
                "baz": "qux",
                "foo": [ "a", 2, "c" ]
           }"#;
        let patches_str = r#"
            [
                { "op": "test", "path": "/baz", "value": "qux" },
                { "op": "test", "path": "/foo/1", "value": 2 }
            ]"#;
        test_json_patch_arr(data, patches_str, data)?;
        Ok(())
    }

    #[test]
    fn test_a_value_error() -> Result<()> {
        let data = r#"{ "baz": "qux" }"#;
        let patches_str = r#"
            [
                { "op": "test", "path": "/baz", "value": "bar" }
            ]
            "#;
        match test_json_patch_arr(data, patches_str, data) {
            Ok(_) => Err(anyhow!("not get test error")),
            Err(e) => match e.downcast_ref::<JsonPatchError>() {
                Some(JsonPatchError::TestFail {
                    json_ptr,
                    expected,
                    actual,
                }) => {
                    if json_ptr.to_escaped_string() == "/baz"
                        && expected.to_string() == "\"bar\""
                        && actual.to_string() == "\"qux\""
                    {
                        Ok(())
                    } else {
                        Err(anyhow!("Wrong test fail error: {}", e))
                    }
                }
                None => Err(anyhow!("Not get JsonPatchError, get {}", e)),
                _ => Err(anyhow!("Get the wrong JsonPatchError {}", e)),
            },
        }
    }

    #[test]
    fn ignore_unrecognized_elements() -> Result<()> {
        let data = r#"{ "foo": "bar" }"#;
        let patch_str = r#"
            [
                { "op": "add", "path": "/baz", "value": "qux", "xyz": 123 }
            ]
            "#;
        let expected_str = r#"
            {
                "foo": "bar",
                "baz": "qux"
            }
            "#;
        test_json_patch_arr(data, patch_str, expected_str)?;
        Ok(())
    }
}
