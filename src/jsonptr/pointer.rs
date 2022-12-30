use std::ops::Deref;
use super::value_mut_ref::ValueMutRef;
use anyhow::{anyhow, Result};
use serde_json::Value;
use super::token::{TokenIndex, Token};

struct JsonPointer {
    tokens: Vec<Token>,
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
                            }
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

    fn get_mut(self, val: &mut Value) -> Result<ValueMutRef> {
        if self.len() == 0 {
            return Ok(ValueMutRef::Root(val));
        }

        let mut cur_ref = val;
        // iteral whole path excpet last one
        for token in &self[0..self.len() - 1] {
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
        // will always not fail, because check the len first;
        let last_token = self.last().unwrap();
        match cur_ref {
            Value::Array(arr) => Ok(ValueMutRef::ArrayElem {
                parent: arr,
                idx: last_token
                    .as_index()
                    .ok_or(anyhow!("not a valid digit index"))?,
            }),
            Value::Object(obj) => Ok(ValueMutRef::ObjElem {
                parent: obj,
                key: last_token.as_key().to_string(),
            }),
            _ => Err(anyhow!("Not an array or an object")),
        }
    }
}

