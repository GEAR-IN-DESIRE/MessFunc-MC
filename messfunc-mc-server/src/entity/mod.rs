use crate::pos::{EntityPos, SyncEntityPos};
use std::any::Any;
use uuid::Uuid;

pub mod player;
mod living;

pub trait EntityTrait: Sync + Send + Any {

}
pub struct Entity {
    pub uuid: Uuid,
}
impl Entity {
    pub fn new(pos: EntityPos) -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }
}