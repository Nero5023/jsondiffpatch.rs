use json_diff_patch::DiffChange;
use json_diff_patch::JsonDiff;
use json_diff_patch::Path;
use json_diff_patch::PathElem;
use serde_json::Value;
use std::unreachable;

fn format_json_val<F>(
    jval: &Value,
    key: Option<String>,
    indent_count: usize,
    diff_op: Option<&str>,
    output: &mut F,
) where
    F: FnMut(&str),
{
    let diff_op_s = diff_op.unwrap_or(" ");
    let prefix = if let Some(key) = key {
        format!("{}{}{}: ", diff_op_s, " ".repeat(indent_count), key)
    } else {
        format!("{}{}", diff_op_s, " ".repeat(indent_count))
    };
    match jval {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            output(&format!("{}{}", prefix, jval))
        }
        Value::Array(arr) => {
            let left_bracket = format!("{}{}", prefix, "[");
            output(&left_bracket);
            arr.iter()
                .for_each(|v| format_json_val(v, None, indent_count + 4, diff_op, output));
            let right_bracket = format!("{}{}", " ".repeat(indent_count), "]");
            output(&right_bracket)
        }
        Value::Object(vmap) => {
            let left_brace = format!("{}{}", prefix, "{");
            output(&left_brace);
            for (key, val) in vmap {
                format_json_val(val, Some(key.to_owned()), indent_count + 4, diff_op, output);
            }
            let right_brace = format!("{}{}{}", diff_op_s, " ".repeat(indent_count), "}");
            output(&right_brace);
        }
    };
}

fn format_json_loop<F>(
    jval: &Value,
    curr_path: &Path,
    json_diffs: &JsonDiff,
    indent_count: usize,
    output: &mut F,
) where
    F: FnMut(&str),
{
    let key = if let Some(last_key) = curr_path.last() {
        match last_key {
            PathElem::Key(key) => Some(key.to_owned()),
            PathElem::Index(_) => None,
        }
    } else {
        None
    };

    let indent_key = if let Some(s) = &key {
        format!(r#"{}{}: "#, " ".repeat(indent_count), s)
    } else {
        " ".repeat(indent_count)
    };

    // added keys
    if let Some(mut parent_path) = curr_path.parent_path() {
        //let parent_path_str = parent_path.to_string();
        if let Some(add_keys) = json_diffs.get_add_keys(&parent_path) {
            for key in add_keys {
                parent_path.push_key(key);
                let val = json_diffs.get_diffchange(&parent_path).unwrap();
                assert!(matches!(val, DiffChange::Add(_)));
                if let DiffChange::Add(new_val) = val {
                    format_json_val(
                        new_val,
                        Some(key.to_string()),
                        indent_count,
                        Some("+"),
                        output,
                    );
                }
                parent_path.pop();
            }
        }
    }

    if let Some(diff_change) = json_diffs.get_diffchange(curr_path) {
        match diff_change {
            DiffChange::Remove(val) => format_json_val(val, key, indent_count, Some("-"), output),
            DiffChange::Replace { old_val, new_val } => {
                format_json_val(old_val, key.to_owned(), indent_count, Some("-"), output);
                format_json_val(new_val, key, indent_count, Some("+"), output);
            }
            DiffChange::Add(val) => format_json_val(val, key, indent_count, Some("+"), output),
        }
    } else {
        match jval {
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                format_json_val(jval, key, indent_count, None, output)
            }
            Value::Object(vmap) => {
                let left_brace = format!("{}{}{}", " ", indent_key, "{");
                output(&left_brace);
                for (key, val) in vmap {
                    let new_path = curr_path.clone_then_add_key(key);
                    // TODO: use static value to + 4
                    format_json_loop(val, &new_path, json_diffs, indent_count + 4, output);
                }
                let right_brace = format!("{}{}{}", " ", " ".repeat(indent_count), "}");
                output(&right_brace)
            }
            Value::Array(arr) => {
                let left_bracket = format!("{}{}{}", " ", indent_key, "[");
                output(&left_bracket);

                let mut cur_idx: usize = 0;
                let mut real_idx: usize = 0;
                let mut curr_len = arr.len();
                while cur_idx < curr_len {
                    let new_path = curr_path.clone_then_add_idx(cur_idx);
                    if let Some(diff_change) = json_diffs.get_diffchange(&new_path) {
                        match diff_change {
                            DiffChange::Add(val) => {
                                format_json_val(val, None, indent_count + 4, Some("+"), output);
                                cur_idx += 1;
                                curr_len += 1;
                            }
                            DiffChange::Remove(val) => {
                                format_json_val(val, None, indent_count + 4, Some("-"), output);
                                assert_eq!(val, &arr[real_idx]);
                                curr_len -= 1;
                                real_idx += 1;
                            }
                            DiffChange::Replace { old_val, new_val } => {
                                format_json_val(old_val, None, indent_count + 4, Some("-"), output);
                                format_json_val(new_val, None, indent_count + 4, Some("+"), output);
                                cur_idx += 1;
                                real_idx += 1;
                            }
                        }
                    } else {
                        format_json_loop(
                            &arr[real_idx],
                            &new_path,
                            json_diffs,
                            indent_count + 4,
                            output,
                        );
                        real_idx += 1;
                        cur_idx += 1;
                    }
                }
                let mut new_path = curr_path.clone_then_add_idx(cur_idx);
                while let Some(diff_change) = json_diffs.get_diffchange(&new_path) {
                    match diff_change {
                        DiffChange::Add(val) => {
                            format_json_val(val, None, indent_count + 4, Some("+"), output);
                            cur_idx += 1;
                            new_path.pop();
                            new_path.push_idx(cur_idx);
                        }
                        DiffChange::Remove(_) => {
                            cur_idx += 1;
                            new_path.pop();
                            new_path.push_idx(cur_idx);
                        }
                        DiffChange::Replace {
                            old_val: _,
                            new_val: _,
                        } => {
                            unreachable!();
                        }
                    }
                }
                let right_bracket = format!("{}{}", " ".repeat(indent_count), "]");
                output(&right_bracket);
            }
        }
    }
}

fn main() {
    let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678",
                "xxx"
            ],
            "c": {
                "a": 1,
                "b": 2
            }
        }"#;
    let data1 = r#"
        {
            "name": "John Doe bill",
            "age": 43,
            "phones": [
                "a",
                "+44 1234567",
                "+44 2345678",
                "yy",
                "zz"
            ],
            "key0": "name1"
        }"#;
    // TODO: check diffs is None
    let json_diffs = JsonDiff::diff_json(data, data1).unwrap();

    let mut output_mut = |line: &str| {
        println!("{}", line);
    };

    let v: Value = serde_json::from_str(data).unwrap();

    format_json_loop(&v, &Path::empty(), &json_diffs, 1, &mut output_mut);
}
