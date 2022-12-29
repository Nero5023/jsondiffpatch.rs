use std::{ops::Deref, thread::current};

use anyhow::{anyhow, Result};
use serde_json::Value;

struct JsonPointer {
    tokens: Vec<Token>,
}

struct Token {
    val: String,
}

impl Deref for JsonPointer {
    type Target = Vec<Token>;

    fn deref(&self) -> &Self::Target {
        &self.tokens
    }
}

impl JsonPointer {
    fn get(self, val: &Value) -> Result<&Value> {
        let mut cur_ref = val;
        for token in self.iter() {
            match cur_ref {
                Value::Array(arr) => {
                    if let Some(token_index) = token.as_index() {
                        match token_index {
                            TokenIndex::Index(idx) => {
                                if idx < arr.len() {
                                    cur_ref = &arr[idx];
                                } else {
                                    return Err(anyhow!("index out of range"));
                                }
                            }
                            TokenIndex::IndexAfterLastElem => {
                                return Err(anyhow!("index out of range"));
                            },
                        }
                    } else {
                        return Err(anyhow!("not a valid digit index"));
                    }
                }
                Value::Object(obj) => {
                    let key = token.as_key();
                    if let Some(child) = obj.get(key) {
                        cur_ref = child;
                    } else {
                        return Err(anyhow!("key {} not exist", key));
                    }
                }
                _ => return Err(anyhow!("Not an array or an object")),
            }
        }
        Ok(cur_ref)
    }

    fn get_mut(self, val: &mut Value) -> Result<&mut Value> {
        let mut cur_ref = val;
        for token in self.iter() {
            match cur_ref {
                Value::Array(arr) => {
                    if let Some(token_index) = token.as_index() {
                        match token_index {
                            TokenIndex::Index(idx) => {
                                if idx < arr.len() {
                                    cur_ref = &mut arr[idx];
                                } else {
                                    return Err(anyhow!("index out of range"));
                                }
                            }
                            TokenIndex::IndexAfterLastElem => todo!(),
                        }
                    } else {
                        return Err(anyhow!("not a valid digit index"));
                    }
                }
                Value::Object(obj) => {
                    let key = token.as_key();
                    if let Some(child) = obj.get_mut(key) {
                        cur_ref = child;
                    } else {
                        return Err(anyhow!("key {} not exist", key));
                    }
                }
                _ => return Err(anyhow!("Not an array or an object")),
            }
        }
        Ok(cur_ref)
    }
}

enum TokenIndex {
    Index(usize),
    IndexAfterLastElem,
}

impl Token {
    fn as_key(&self) -> &str {
        &self.val
    }

    // TODO: maybe use Result for this, and use self defined error
    fn as_index(&self) -> Option<TokenIndex> {
        if self.val == "-" {
            return Some(TokenIndex::IndexAfterLastElem);
        }
        if self.val.len() != 1 {
            if self.val.trim_start_matches('0').len() != self.val.len() {
                // Leading zero
                return None;
            }
        }
        if let Ok(index) = self.val.parse::<usize>() {
            Some(TokenIndex::Index(index))
        } else {
            None
        }
    }
}
