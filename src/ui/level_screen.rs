use std::path::Path;

use component_group::ComponentGroup;
use sdl2::{rect::Point, render::RenderTarget};
use specs::{Dispatcher, Entities, Entity, Join, Read, ReadStorage, World, WorldExt};

use crate::components::{Player, PlayerComponents, Position, Stairs};
use crate::generator::GenLevel;
use crate::map::FloorMap;
use crate::resources::{ActionQueue, ChangeGameState, Event, EventQueue, FramesElapsed, GameState};

use super::debug;
use super::renderer::{render_player_visible, RenderContext};
use super::SDLError;

pub struct LevelScreen<'a, 'b> {
    dispatcher: Dispatcher<'a, 'b>,
    world: World,
}

impl<'a, 'b> From<GenLevel<'a, 'b>> for LevelScreen<'a, 'b> {
    fn from(GenLevel { dispatcher, world }: GenLevel<'a, 'b>) -> Self {
        Self { dispatcher, world }
    }
}

impl<'a, 'b> LevelScreen<'a, 'b> {
    /// Returns the components of the player on this level
    pub fn player_components(&self) -> PlayerComponents {
        PlayerComponents::first_from_world(&self.world)
            .expect("bug: expected player to be in world")
            .1
    }

    /// Finds the position next to the ToNextLevel gate with the given ID
    pub fn find_to_next_level_adjacent(&self, gate_id: usize) -> Point {
        let (positions, stairs) = self
            .world
            .system_data::<(ReadStorage<'_, Position>, ReadStorage<'_, Stairs>)>();
        let pos = (&positions, &stairs)
            .join()
            .find_map(|(&Position(pos), stairs)| match stairs {
                Stairs::ToNextLevel { id } if *id == gate_id => Some(pos),
                _ => None,
            })
            .expect("bug: could not find next level gate with matching ID");

        // Find the empty position adjacent to this staircase. There should only be one.
        let map = self.world.system_data::<Read<'_, FloorMap>>();
        let tile_pos = map.world_to_tile_pos(pos);
        let empty = map
            .grid()
            .adjacent_positions(tile_pos)
            .find(|&p| !map.grid().get(p).is_wall())
            .expect("bug: should be one empty position adjacent to a staircase");
        empty.center(map.tile_size() as i32)
    }

    /// Finds the position next to the ToPrevLevel gate with the given ID
    pub fn find_to_prev_level_adjacent(&self, gate_id: usize) -> Point {
        let (positions, stairs) = self
            .world
            .system_data::<(ReadStorage<'_, Position>, ReadStorage<'_, Stairs>)>();
        let pos = (&positions, &stairs)
            .join()
            .find_map(|(&Position(pos), stairs)| match stairs {
                Stairs::ToPrevLevel { id } if *id == gate_id => Some(pos),
                _ => None,
            })
            .expect("bug: could not find previous level gate with matching ID");

        // Find the empty position adjacent to this staircase. There should only be one.
        let map = self.world.system_data::<Read<'_, FloorMap>>();
        let tile_pos = map.world_to_tile_pos(pos);
        let empty = map
            .grid()
            .adjacent_positions(tile_pos)
            .find(|&p| !map.grid().get(p).is_wall())
            .expect("bug: should be one empty position adjacent to a staircase");
        empty.center(map.tile_size() as i32)
    }

    /// Updates the player entity on this level
    pub fn update_player(&mut self, player: PlayerComponents) {
        match self.player_entity() {
            Some(player_entity) => player
                .update(&mut self.world, player_entity)
                .expect("bug: failed to update player when changing levels"),
            None => {
                player.create(&mut self.world);
            }
        }
    }

    /// Gets the entity of the player on this level or None if a player hasn't been created yet
    fn player_entity(&self) -> Option<Entity> {
        let (entities, players) = self
            .world
            .system_data::<(Entities<'_>, ReadStorage<'_, Player>)>();
        let mut player_iter = (&entities, &players).join();
        let player_entity = player_iter.next().map(|(entity, _)| entity);
        player_iter
            .next()
            .map(|_| unreachable!("bug: more than one player in world"));
        player_entity
    }

    /// Dispatch the given events and update the state based on the frames that have elapsed
    pub fn dispatch(
        &mut self,
        frames_elapsed: FramesElapsed,
        events: Vec<Event>,
    ) -> Option<GameState> {
        //NOTE: All resources here must already be added when the world is created
        *self.world.write_resource::<FramesElapsed>() = frames_elapsed;
        *self.world.write_resource::<ChangeGameState>() = ChangeGameState::default();
        *self.world.write_resource::<ActionQueue>() = ActionQueue::default();
        *self.world.write_resource::<EventQueue>() = EventQueue(events);

        self.dispatcher.dispatch(&mut self.world);
        World::maintain(&mut self.world);

        let state = self.world.system_data::<Read<'_, ChangeGameState>>();
        state.get()
    }

    /// Render the entire state of the level (the entire map) to the given filename.
    ///
    /// Useful for debugging. This function is fairly "slow", so use sparingly.
    pub fn render_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SDLError> {
        let map = self.world.system_data::<Read<'_, FloorMap>>();
        debug::render_to_file(&map, &self.world, path)
    }

    pub fn render<T: RenderTarget>(&self, ctx: &mut RenderContext<T>) -> Result<(), SDLError> {
        render_player_visible(self.world.system_data(), ctx)
    }
}
