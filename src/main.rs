use anyhow::Result;
use clap::Parser;
use console::Style;
use jsondiff::DiffChange;
use jsondiff::JsonDiff;
use jsondiff::Path;
use serde_json::Value;
use std::fs;
use std::process;
use std::unreachable;

const INDENT_SIZE: usize = 4;

fn format_json_val<F>(
    jval: &Value,
    key: Option<String>,
    indent_count: usize,
    diff_op: Option<&str>,
    output: &mut F,
) where
    F: FnMut(&str, &str),
{
    let diff_op_s = diff_op.unwrap_or(" ");
    let prefix = if let Some(key) = key {
        format!("{}{}{}: ", diff_op_s, " ".repeat(indent_count), key)
    } else {
        format!("{}{}", diff_op_s, " ".repeat(indent_count))
    };
    match jval {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            output(diff_op_s, &format!("{}{}", prefix, jval))
        }
        Value::Array(arr) => {
            let left_bracket = format!("{}{}", prefix, "[");
            output(diff_op_s, &left_bracket);
            arr.iter().for_each(|v| {
                format_json_val(v, None, indent_count + INDENT_SIZE, diff_op, output)
            });
            let right_bracket = format!("{}{}", " ".repeat(indent_count), "]");
            output(diff_op_s, &right_bracket)
        }
        Value::Object(vmap) => {
            let left_brace = format!("{}{}", prefix, "{");
            output(diff_op_s, &left_brace);
            for (key, val) in vmap {
                format_json_val(
                    val,
                    Some(key.to_owned()),
                    indent_count + INDENT_SIZE,
                    diff_op,
                    output,
                );
            }
            let right_brace = format!("{}{}{}", diff_op_s, " ".repeat(indent_count), "}");
            output(diff_op_s, &right_brace);
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
    F: FnMut(&str, &str),
{
    let key = curr_path.current_key();
    let indent_key = if let Some(s) = &key {
        // e.g. for path /a/b/c
        // ______c:
        format!(r#"{}{}: "#, " ".repeat(indent_count), s)
    } else {
        // for arr index path or empty path, just indent space
        " ".repeat(indent_count)
    };

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
                output(" ", &left_brace);

                {
                    // added keys
                    let mut path = curr_path.clone();
                    if let Some(add_keys) = json_diffs.get_add_keys(&path) {
                        for key in add_keys {
                            path.push_key(key);
                            let val = json_diffs.get_diffchange(&path).unwrap();
                            assert!(matches!(val, DiffChange::Add(_)));
                            if let DiffChange::Add(new_val) = val {
                                format_json_val(
                                    new_val,
                                    Some(key.to_string()),
                                    indent_count + INDENT_SIZE,
                                    Some("+"),
                                    output,
                                );
                            }
                            path.pop();
                        }
                    }
                }

                for (key, val) in vmap {
                    let new_path = curr_path.clone_then_add_key(key);
                    format_json_loop(
                        val,
                        &new_path,
                        json_diffs,
                        indent_count + INDENT_SIZE,
                        output,
                    );
                }
                let right_brace = format!("{}{}{}", " ", " ".repeat(indent_count), "}");
                output(" ", &right_brace)
            }
            Value::Array(arr) => {
                let left_bracket = format!("{}{}{}", " ", indent_key, "[");
                output(" ", &left_bracket);

                let mut cur_idx: usize = 0;
                let mut real_idx: usize = 0;
                let mut curr_len = arr.len();
                while cur_idx < curr_len {
                    let new_path = curr_path.clone_then_add_idx(cur_idx);
                    if let Some(diff_change) = json_diffs.get_diffchange(&new_path) {
                        match diff_change {
                            DiffChange::Add(val) => {
                                format_json_val(
                                    val,
                                    None,
                                    indent_count + INDENT_SIZE,
                                    Some("+"),
                                    output,
                                );
                                cur_idx += 1;
                                curr_len += 1;
                            }
                            DiffChange::Remove(val) => {
                                format_json_val(
                                    val,
                                    None,
                                    indent_count + INDENT_SIZE,
                                    Some("-"),
                                    output,
                                );
                                assert_eq!(val, &arr[real_idx]);
                                curr_len -= 1;
                                real_idx += 1;
                            }
                            DiffChange::Replace { old_val, new_val } => {
                                format_json_val(
                                    old_val,
                                    None,
                                    indent_count + INDENT_SIZE,
                                    Some("-"),
                                    output,
                                );
                                format_json_val(
                                    new_val,
                                    None,
                                    indent_count + INDENT_SIZE,
                                    Some("+"),
                                    output,
                                );
                                cur_idx += 1;
                                real_idx += 1;
                            }
                        }
                    } else {
                        format_json_loop(
                            &arr[real_idx],
                            &new_path,
                            json_diffs,
                            indent_count + INDENT_SIZE,
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
                            format_json_val(
                                val,
                                None,
                                indent_count + INDENT_SIZE,
                                Some("+"),
                                output,
                            );
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
                output(" ", &right_bracket);
            }
        }
    }
}

fn read_json_file(path: &str) -> String {
    let res = fs::read_to_string(path);
    match res {
        Ok(content) => {
            let json: serde_json::Result<Value> = serde_json::from_str(&content);
            if let Err(err) = json {
                println!("Json `{}` parse error: {}", path, err);
                process::exit(1);
            }
            content
        }
        Err(err) => {
            println!("{}: {}", path, err);
            process::exit(1);
        }
    }
}

#[derive(Parser)]
struct Cli {
    /// Old file
    first_json: String,
    /// New file
    second_json: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let json1 = read_json_file(&args.first_json);
    let json2 = read_json_file(&args.second_json);

    let json_diffs = JsonDiff::diff_json(&json1, &json2)?;

    let mut output_mut = |diff_opp: &str, line: &str| {
        let str_output = match diff_opp {
            "+" => format!("{}", Style::new().green().apply_to(line)),
            "-" => format!("{}", Style::new().red().apply_to(line)),
            _ => line.to_owned(),
        };
        println!("{}", str_output);
    };

    let v: Value = serde_json::from_str(&json1)?;

    format_json_loop(&v, &Path::empty(), &json_diffs, 1, &mut output_mut);
    Ok(())
}
