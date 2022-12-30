use super::pointer::JsonPointer;
use anyhow::Result;
use serde_json::Value;

pub trait OperateByJsonptr {
    fn get(&self, ptr: JsonPointer) -> Result<&Value>;
    fn add(&mut self, ptr: JsonPointer, val: Value) -> Result<()>;
    fn replace(&mut self, ptr: JsonPointer, val: Value) -> Result<()>;
    fn delete(&mut self, ptr: JsonPointer) -> Result<()>;
}

impl OperateByJsonptr for Value {
    fn get(&self, ptr: JsonPointer) -> Result<&Value> {
        ptr.get(val)
    }

    fn add(&mut self, ptr: JsonPointer, val: Value) -> Result<()> {
        let mut_ref = ptr.get_mut(self)?;
        mut_ref.add(val)
    }

    fn replace(&mut self, ptr: JsonPointer, val: Value) -> Result<()> {
        let mut_ref = ptr.get_mut(self)?;
        mut_ref.replace(val)
    }

    fn delete(&mut self, ptr: JsonPointer) -> Result<()> {
        let mut_ref = ptr.get_mut(self)?;
        mut_ref.delete()
    }
}
