#[cfg(test)]
mod tests {

    use anyhow::anyhow;
    use anyhow::Result;
    use serde_json::json;
    use serde_json::Value;
    use json_diff_patch::{Path, PathElem};
    use json_diff_patch::patch::{JsonPatch, JsonPatchError, Patch, PatchElem};

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
                         path,
                         expected,
                         actual,
                     }) => {
                    if path.to_string() == "/baz"
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

    #[test]
    fn add_to_nonexistent_target()->Result<()>{
        let data = r#"{ "foo": "bar" }"#;
        let patch_str = r#"
           [
                { "op": "add", "path": "/baz/bat", "value": "qux" }
           ]
        "#;
        match test_json_patch_arr(data,patch_str,data){
            Ok(_) => Err(anyhow!("not get test error")),
            Err(e) => {
                println!("######{}",e);
                match e.downcast_ref::<JsonPatchError>() {
                    Some(JsonPatchError::ParentNodeNotExist) => Ok(()),
                    None => Err(anyhow!("Not get JsonPatchError, get {}", e)),
                    _ => Err(anyhow!("Get the wrong JsonPatchError {}", e)),
                }},
        }
    }
}