use std::any::TypeId;

pub struct Undefined();
pub type NodeId = (TypeId, usize, usize);
