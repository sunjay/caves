use std::path::Path;

use sdl2::{
    image::{SaveSurface},
    pixels::{PixelFormatEnum},
    surface::Surface,
    rect::Point,
    render::{Canvas, RenderTarget},
};
use specs::{Dispatcher, World, Join, Entity, Entities, ReadStorage};
use component_group::ComponentGroup;

use crate::assets::{AssetManager, TextureManager, SpriteManager};
use crate::map_sprites::MapSprites;
use crate::generator::GenLevel;
use crate::map::FloorMap;
use crate::components::{PlayerComponents, Player, Position, Stairs};
use crate::resources::{FramesElapsed, Event, ChangeGameState, GameState, ActionQueue, EventQueue};

use super::renderer::{RenderData, render_area, render_player_visible};
use super::SDLError;

pub struct LevelScreen<'a, 'b> {
    dispatcher: Dispatcher<'a, 'b>,
    world: World,
}

impl<'a, 'b> From<GenLevel<'a, 'b>> for LevelScreen<'a, 'b> {
    fn from(GenLevel {dispatcher, world}: GenLevel<'a, 'b>) -> Self {
        Self {dispatcher, world}
    }
}

impl<'a, 'b> LevelScreen<'a, 'b> {
    /// Returns the components of the player on this level
    pub fn player_components(&self) -> PlayerComponents {
        PlayerComponents::first_from_world(&self.world)
            .expect("bug: expected player to be in world").1
    }

    /// Finds the position next to the ToNextLevel gate with the given ID
    pub fn find_to_next_level_adjacent(&self, gate_id: usize) -> Point {
        let (positions, stairs) = self.world.system_data::<(ReadStorage<Position>, ReadStorage<Stairs>)>();
        let pos = (&positions, &stairs).join().find_map(|(&Position(pos), stairs)| match stairs {
            Stairs::ToNextLevel {id} if *id == gate_id => Some(pos),
            _ => None,
        }).expect("bug: could not find next level gate with matching ID");

        // Find the empty position adjacent to this staircase. There should only be one.
        let map = self.world.read_resource::<FloorMap>();
        let tile_pos = map.world_to_tile_pos(pos);
        let empty = map.grid().adjacent_positions(tile_pos).find(|&p| !map.grid().get(p).is_wall())
            .expect("bug: should be one empty position adjacent to a staircase");
        empty.center(map.tile_size() as i32)
    }

    /// Finds the position next to the ToPrevLevel gate with the given ID
    pub fn find_to_prev_level_adjacent(&self, gate_id: usize) -> Point {
        let (positions, stairs) = self.world.system_data::<(ReadStorage<Position>, ReadStorage<Stairs>)>();
        let pos = (&positions, &stairs).join().find_map(|(&Position(pos), stairs)| match stairs {
            Stairs::ToPrevLevel {id} if *id == gate_id => Some(pos),
            _ => None,
        }).expect("bug: could not find previous level gate with matching ID");

        // Find the empty position adjacent to this staircase. There should only be one.
        let map = self.world.read_resource::<FloorMap>();
        let tile_pos = map.world_to_tile_pos(pos);
        let empty = map.grid().adjacent_positions(tile_pos).find(|&p| !map.grid().get(p).is_wall())
            .expect("bug: should be one empty position adjacent to a staircase");
        empty.center(map.tile_size() as i32)
    }

    /// Updates the player entity on this level
    pub fn update_player(&mut self, player: PlayerComponents) {
        match self.player_entity() {
            Some(player_entity) => player.update(&mut self.world, player_entity)
                .expect("bug: failed to update player when changing levels"),
            None => {player.create(&mut self.world);},
        }
    }

    /// Gets the entity of the player on this level or None if a player hasn't been created yet
    fn player_entity(&self) -> Option<Entity> {
        let (entities, players) = self.world.system_data::<(Entities, ReadStorage<Player>)>();
        let mut player_iter = (&entities, &players).join();
        let player_entity = player_iter.next().map(|(entity, _)| entity);
        player_iter.next().map(|_| unreachable!("bug: more than one player in world"));
        player_entity
    }

    /// Dispatch the given events and update the state based on the frames that have elapsed
    pub fn dispatch(&mut self, frames_elapsed: FramesElapsed, events: Vec<Event>) -> Option<GameState> {
        //NOTE: All resources here must already be added when the world is created
        *self.world.write_resource() = frames_elapsed;
        *self.world.write_resource() = ChangeGameState::default();
        *self.world.write_resource() = ActionQueue::default();
        *self.world.write_resource() = EventQueue(events);

        self.dispatcher.dispatch(&mut self.world.res);

        // Register any updates
        self.world.maintain();

        // Return any changes of game state that have been requested
        self.world.read_resource::<ChangeGameState>().get()
    }

    /// Render the entire state of the level (the entire map) to the given filename.
    ///
    /// Useful for debugging. This function is fairly "slow", so use sparingly.
    pub fn render_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SDLError> {
        //TODO: This code is super fragile. It relies on the SpriteIds generated by the main
        // asset manager corresponding to the asset manager declared here. This only works now
        // because we happen to use the same constructor. If the code there or the code here
        // changes, this assumption could be *very* wrong.

        let map = self.world.read_resource::<FloorMap>();

        let level_boundary = map.level_boundary();
        let mut canvas = Surface::new(level_boundary.width(), level_boundary.height(),
            PixelFormatEnum::RGBA8888).and_then(|c| c.into_canvas()).map_err(SDLError)?;
        let texture_creator = canvas.texture_creator();

        let tile_size = 16;
        let AssetManager {
            textures,
            map_sprites,
            sprites,
            ..
        } = AssetManager::load(&texture_creator, 30, tile_size)?;

        let data: RenderData = self.world.system_data();
        render_area(data, level_boundary, &mut canvas, &map_sprites, &textures,
            &sprites, |_, _| true)?;

        canvas.into_surface().save(path).map_err(SDLError)?;
        Ok(())
    }

    pub fn render<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        textures: &TextureManager<<T as RenderTarget>::Context>,
        sprites: &SpriteManager,
        map_sprites: &MapSprites,
    ) -> Result<(), SDLError> {
        render_player_visible(self.world.system_data(), canvas, textures, sprites, map_sprites)
    }
}
