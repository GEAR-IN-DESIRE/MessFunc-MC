use crate::entity::Entity;
use crate::pos::EntityPos;

pub struct LivingEntity {
    pub entity: Entity,
}

impl LivingEntity {
    pub fn new(pos: EntityPos) -> LivingEntity {
        LivingEntity {
            entity: Entity::new(pos),
        }
    }
}