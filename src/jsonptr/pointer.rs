use super::token::{Token, TokenIndex};
use super::value_mut_ref::ValueMutRef;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::ops::Deref;

pub struct JsonPointer {
    tokens: Vec<Token>,
}

impl Deref for JsonPointer {
    type Target = Vec<Token>;

    fn deref(&self) -> &Self::Target {
        &self.tokens
    }
}

impl JsonPointer {
    pub fn get(self, val: &Value) -> Result<&Value> {
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

    pub fn get_mut(self, val: &mut Value) -> Result<ValueMutRef> {
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

impl JsonPointer {
    fn parse(s: &str) -> Result<Self> {
        if s == "" {
            // Empty tokens
            return Ok(JsonPointer { tokens: vec![] });
        }
        if !s.starts_with('/') {
            return Err(anyhow!("Path is not start with '/'"));
        }
        let tokens = s
            .split('/')
            .skip(1) // skip for first leaing empty elem
            .map(|s| Token::new(s))
            .collect::<Vec<Token>>();

        Ok(JsonPointer { tokens })
    }

    pub fn new(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl TryFrom<&str> for JsonPointer {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        JsonPointer::new(s)
    }
}

impl TryFrom<String> for JsonPointer {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        TryFrom::try_from(s.as_ref())
    }
}
