use crate::entity::living::LivingEntity;
use crate::entity::EntityTrait;
use crate::net::client::Client;
use crate::pos::EntityPos;
use uuid::Uuid;

#[derive()]
pub struct Player {
    pub living_entity: LivingEntity,
    pub client: Client,
}
impl Player {
    pub fn new(client: Client) -> Player {
        let pos = EntityPos {
            world: Uuid::new_v4(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        Player {
            living_entity: LivingEntity::new(pos),
            client,
        }
    }
    
    pub fn send_message(&mut self, message: &str) {
        self.client.send_packet(message.as_bytes());
        println!("message: {:?}: {}", self.living_entity.entity.uuid, message);
    }
}

impl EntityTrait for Player {}
