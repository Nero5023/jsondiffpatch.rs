use serde_json::{Map, Value};
use anyhow::{Result, anyhow};
use super::token::TokenIndex;


pub enum ValueMutRef<'a> {
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
    pub fn set(self, val: Value) -> Result<()> {
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

    pub fn add(self, val: Value) -> Result<()> {
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

    pub fn replace(self, val: Value) -> Result<()> {
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

    pub fn delete(self) -> Result<()> {
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

    pub fn get(&self) -> Option<&Value> {
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
