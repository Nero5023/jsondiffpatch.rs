use serde_json::Map;
use std::ops::Deref;

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

enum ValueMutRef<'a> {
    ArrayElem {
        parent: &'a mut Vec<Value>,
        idx: TokenIndex,
    },
    // TODO: check if 'a for key is correct
    ObjElem {
        parent: &'a mut Map<String, Value>,
        key: String,
    },
    Root(&'a mut Value),
}

impl<'a> ValueMutRef<'a> {
    fn set(self, val: Value) -> Result<()> {
        match self {
            ValueMutRef::ArrayElem { parent, idx } => match idx {
                TokenIndex::Index(idx) => {
                    if idx < parent.len() {
                        parent[idx] = val;
                        Ok(())
                    } else {
                        Err(anyhow!("Index out of range"))
                    }
                }
                TokenIndex::IndexAfterLastElem => {
                    parent.push(val);
                    Ok(())
                }
            },
            ValueMutRef::ObjElem { parent, key } => {
                parent[&key] = val;
                Ok(())
            }
            ValueMutRef::Root(root) => {
                *root = val;
                Ok(())
            }
        }
    }

    fn add(self, val: Value) -> Result<()> {
        match self {
            ValueMutRef::ArrayElem { parent, idx } => match idx {
                TokenIndex::Index(idx) => {
                    if idx <= parent.len() {
                        parent.insert(idx, val);
                        Ok(())
                    } else {
                        Err(anyhow!("Index out of range"))
                    }
                }
                TokenIndex::IndexAfterLastElem => {
                    parent.push(val);
                    Ok(())
                }
            },
            _ => self.set(val),
        }
    }

    fn replace(self, val: Value) -> Result<()> {
        match self {
            ValueMutRef::ObjElem { parent, key } => {
                if parent.contains_key(&key) {
                    parent[&key] = val;
                    Ok(())
                } else {
                    Err(anyhow!("key {} not exist", key))
                }
            }
            _ => self.set(val),
        }
    }

    fn delete(self) -> Result<()> {
        match self {
            ValueMutRef::ArrayElem { parent, idx } => {
                match idx {
                    TokenIndex::Index(idx) => {
                        if idx < parent.len() {
                            parent.remove(idx);
                            Ok(())
                        } else {
                            Err(anyhow!("Index out of range"))
                        }
                    }
                    TokenIndex::IndexAfterLastElem => {
                        if parent.len() != 0 {
                            parent.pop();
                        }
                        // TODO: do not know if arr is empty what todo, the current logic means
                        // delete last element of array, if not have last element just ignore.
                        Ok(())
                    }
                }
            }
            ValueMutRef::ObjElem { parent, key } => {
                if parent.contains_key(&key) {
                    parent.remove(&key);
                    Ok(())
                } else {
                    Err(anyhow!("key {} not exist", key))
                }
            }
            ValueMutRef::Root(_) => Err(anyhow!("Cannot delete root")),
        }
    }

    fn get(&self) -> Option<&Value> {
        match self {
            ValueMutRef::ArrayElem { parent, idx } => {
                match idx {
                    TokenIndex::Index(idx) => parent.get(*idx),
                    TokenIndex::IndexAfterLastElem => None,
                }
            },
            ValueMutRef::ObjElem { parent, key } => {
                parent.get(key)
            },
            ValueMutRef::Root(val) => Some(val),
        }
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
