use super::token::TokenIndex;
use anyhow::{anyhow, Result};
use serde_json::{Map, Value};

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
    // set/add/replace/delete use self, take the onwership here, the reason here is that because it
    // will change the mut reference, if use &self, it can still operate mut operations, it will
    // make some confuse
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
                parent.insert(key, val);
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

    pub fn delete(self) -> Result<Value> {
        match self {
            ValueMutRef::ArrayElem { parent, idx } => {
                match idx {
                    TokenIndex::Index(idx) => {
                        if idx < parent.len() {
                            let v = parent.remove(idx);
                            Ok(v)
                        } else {
                            Err(anyhow!("Index out of range"))
                        }
                    }
                    TokenIndex::IndexAfterLastElem => {
                        if parent.len() != 0 {
                            Ok(parent.pop().unwrap())
                        } else {
                            // TODO: do not know if arr is empty what todo, maybe need raise error here
                            todo!()
                        }
                    }
                }
            }
            ValueMutRef::ObjElem { parent, key } => {
                if parent.contains_key(&key) {
                    let v = parent.remove(&key).unwrap();
                    Ok(v)
                } else {
                    Err(anyhow!("key {} not exist", key))
                }
            }
            ValueMutRef::Root(_) => Err(anyhow!("Cannot delete root")),
        }
    }

    pub fn get(&self) -> Option<&Value> {
        match self {
            ValueMutRef::ArrayElem { parent, idx } => match idx {
                TokenIndex::Index(idx) => parent.get(*idx),
                TokenIndex::IndexAfterLastElem => None,
            },
            ValueMutRef::ObjElem { parent, key } => parent.get(key),
            ValueMutRef::Root(val) => Some(val),
        }
    }
}
