mod cell_dweller;
mod movement_system;
mod mining;
mod mining_system;
mod physics_system;
mod recv_system;

use std::collections::vec_deque::VecDeque;
use grid::{GridPoint3, Dir};
use ::movement::TurnDir;
use ::net::{SendMessage, RecvMessage};

pub use ::AutoResource;
pub use self::cell_dweller::CellDweller;
pub use self::movement_system::{MovementSystem, MovementEvent, MovementInputAdapter};
pub use self::mining_system::{MiningSystem, MiningEvent, MiningInputAdapter};
pub use self::physics_system::PhysicsSystem;
pub use self::recv_system::RecvSystem;

use shred;
use specs;

/// `World`-global resource for finding the current cell-dwelling entity being controlled
/// by the player, if any.
///
/// TODO: make this a more general "controlled entity" somewhere?
pub struct ActiveCellDweller {
    pub maybe_entity: Option<specs::Entity>,
}

impl ActiveCellDweller {
    // TODO: replace with AutoResource
    pub fn ensure_registered(world: &mut specs::World) {
        let res_id = shred::ResourceId::new::<ActiveCellDweller>();
        if !world.res.has_value(res_id) {
            world.add_resource(ActiveCellDweller { maybe_entity: None });
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum CellDwellerMessage {
    SetPos(SetPosMessage),
    TryPickUpBlock(TryPickUpBlockMessage),
    RemoveBlock(RemoveBlockMessage),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct SetPosMessage {
    pub entity_id: u64,
    pub new_pos: GridPoint3,
    pub new_dir: Dir,
    pub new_last_turn_bias: TurnDir,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct TryPickUpBlockMessage {
    // TODO:
    // pub globe_entity_id: u64,
    pub cd_entity_id: u64,
    // TODO: what are you trying to pick up? Until we hook that up,
    // just use whatever the server thinks is in front of you.
    // pub pos: GridPoint3,
    // TODO: also include the cell dweller's current position.
    // We'll trust that if it's close enough, so that we don't
    // have to worry about missing out on a position update and
    // picking up a different block than what the client believed
    // they were pickng up!
}

// TODO: this shouldn't really even be a cell dweller message;
// it's a more general thing. But it's also not how we want to
// represent the concept of chunk changes long-term, anyway,
// so just leave it in here for now. Hoo boy, lotsa refactoring
// lies ahead.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct RemoveBlockMessage {
    // TODO: identify the globe.
    // But for that, the server will need to communicate the globe's
    // identity etc. to the client when they join.
    // For now it's just going to find the first globe it can... :)
    // pub globe_entity_id: u64,

    // Don't send it as a "PosInOwningRoot", because we can't trust
    // clients like that.
    //
    // TODO: We should actually be validating EVERYTHING that comes
    // in as a network message.
    pub pos: GridPoint3,
}

/// `World`-global resource for outbound cell-dweller network messages.
pub struct SendMessageQueue {
    // We don't want to queue up any messages unless there's
    // actually a network system hanging around to consume them.
    // TODO: there's got to be a better way to do this.
    // I'm thinking some kind of simple pubsub, that doesn't
    // know anything about atomics/thread synchronisation,
    // but is instead just a dumb collection of `VecDeque`s.
    // As you add more of these, either find something that works
    // or make that thing you described above.
    pub has_consumer: bool,
    pub queue: VecDeque<SendMessage<CellDwellerMessage>>,
}

impl ::AutoResource for SendMessageQueue {
    fn new(_world: &mut specs::World) -> SendMessageQueue {
        SendMessageQueue {
            has_consumer: false,
            queue: VecDeque::new(),
        }
    }
}

/// `World`-global resource for inbound cell-dweller network messages.
pub struct RecvMessageQueue {
    pub queue: VecDeque<RecvMessage<CellDwellerMessage>>,
}

impl ::AutoResource for RecvMessageQueue {
    fn new(_world: &mut specs::World) -> RecvMessageQueue {
        RecvMessageQueue {
            queue: VecDeque::new(),
        }
    }
}
