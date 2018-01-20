use std::sync::mpsc;
use specs;
use specs::{Fetch, LazyUpdate, Entities, ReadStorage, WriteStorage};
use slog::Logger;
use piston::input::Input;

use pk::cell_dweller::{
    CellDweller,
    ActiveCellDweller,
};
use pk::types::*;
use pk::input_adapter;
use pk::Spatial;

use super::grenade::shoot_grenade;
use ::fighter::Fighter;

pub struct ShootInputAdapter {
    sender: mpsc::Sender<ShootEvent>,
}

impl ShootInputAdapter {
    pub fn new(sender: mpsc::Sender<ShootEvent>) -> ShootInputAdapter {
        ShootInputAdapter { sender: sender }
    }
}

impl input_adapter::InputAdapter for ShootInputAdapter {
    fn handle(&self, input_event: &Input) {
        use piston::input::{Button, ButtonState};
        use piston::input::keyboard::Key;

        if let &Input::Button(button_args) = input_event {
            if let Button::Keyboard(key) = button_args.button {
                let is_down = match button_args.state {
                    ButtonState::Press => true,
                    ButtonState::Release => false,
                };
                match key {
                    Key::Space => self.sender.send(ShootEvent(is_down)).unwrap(),
                    _ => (),
                }
            }
        }
    }
}

pub struct ShootEvent(bool);

pub struct ShootSystem {
    input_receiver: mpsc::Receiver<ShootEvent>,
    log: Logger,
    shoot: bool,
}

impl ShootSystem {
    pub fn new(
        world: &mut specs::World,
        input_receiver: mpsc::Receiver<ShootEvent>,
        parent_log: &Logger,
    ) -> ShootSystem {
        ActiveCellDweller::ensure_registered(world);

        ShootSystem {
            input_receiver: input_receiver,
            log: parent_log.new(o!()),
            shoot: false,
        }
    }

    fn consume_input(&mut self) {
        loop {
            match self.input_receiver.try_recv() {
                Ok(ShootEvent(b)) => self.shoot = b,
                Err(_) => return,
            }
        }
    }
}

impl<'a> specs::System<'a> for ShootSystem {
    type SystemData = (
        Fetch<'a, TimeDeltaResource>,
        Entities<'a>,
        Fetch<'a, LazyUpdate>,
        ReadStorage<'a, CellDweller>,
        Fetch<'a, ActiveCellDweller>,
        WriteStorage<'a, Spatial>,
        WriteStorage<'a, Fighter>,
    );

    fn run(&mut self, data: Self::SystemData) {
        self.consume_input();
        let (
            dt,
            entities,
            updater,
            cell_dwellers,
            active_cell_dweller_resource,
            spatials,
            mut fighters,
        ) = data;

        // Find the active fighter, even if we're not currently trying to shoot;
        // we might need to count down the time until we can next shoot.
        let active_cell_dweller_entity = match active_cell_dweller_resource.maybe_entity {
            Some(entity) => entity,
            None => {
                warn!(self.log, "Trying to shoot without an active CellDweller");
                return
            },
        };
        // Assume it is a fighter, because those are the only cell dwellers
        // you're allowed to control in this game.
        let active_fighter = fighters.get_mut(active_cell_dweller_entity).expect("Cell dweller should have had a fighter attached!");

        // Count down until we're allowed to shoot next.
        if active_fighter.seconds_until_next_shot > 0.0 {
            active_fighter.seconds_until_next_shot = (active_fighter.seconds_until_next_shot - dt.0).max(0.0);
        }
        let still_waiting_to_shoot = active_fighter.seconds_until_next_shot > 0.0;

        if self.shoot && ! still_waiting_to_shoot{
            self.shoot = false;

            // TODO: send this as a network message instead:

            // Place the bullet in the same location as the player,
            // relative to the same globe.
            info!(self.log, "Fire!");

            shoot_grenade(
                &entities,
                &updater,
                &cell_dwellers,
                active_cell_dweller_entity,
                &spatials,
                &self.log,
            );
            // Reset time until we can shoot again.
            active_fighter.seconds_until_next_shot = active_fighter.seconds_between_shots;
        }
    }
}
