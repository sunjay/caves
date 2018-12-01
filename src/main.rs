#![deny(unused_must_use)]

#[macro_use]
extern crate specs_derive;
#[macro_use]
extern crate shred_derive;
#[macro_use]
extern crate lazy_static;

extern crate sdl2;
extern crate specs;
extern crate shred;
extern crate rand;
extern crate base64;
extern crate colored;
extern crate rayon;

mod systems;
mod components;
mod generator;
mod resources;
mod map;
mod ui;
mod map_sprites;
mod assets;

use std::{
    thread,
    time::Duration,
};

use sdl2::{
    event::Event as SDLEvent,
    keyboard::Keycode,
};
use specs::{
    Builder,
    DispatcherBuilder,
    World,
};

use components::{
    Position,
    HealthPoints,
    Movement,
    BoundingBox,
    KeyboardControlled,
    CameraFocus,
    Sprite,
    AnimationManager,
    Player,
};
use assets::{TextureManager, SpriteManager};
use resources::{FramesElapsed, ActionQueue, EventQueue, Event, Key};
use ui::{Window, GameScreen, SDLError};
use generator::{GameGenerator, GenGame};
use map_sprites::MapSprites;

fn game_generator(tile_size: u32, map_sprites: &MapSprites) -> GameGenerator {
    GameGenerator {
        attempts: 2000,
        levels: 10,
        rows: 40,
        cols: 50,
        tile_size,
        rooms: (6, 9).into(),
        room_rows: (7, 14).into(),
        room_cols: (8, 16).into(),
        max_overlap: 0.35,
        doors: (1, 3).into(),
        next_prev_tiles: 2,
        sprites: map_sprites,
    }
}

fn main() -> Result<(), SDLError> {
    let fps = 30.0;

    let mut window = Window::init(320, 240)?;
    let texture_creator = window.texture_creator();
    let mut textures = TextureManager::new(&texture_creator);
    let mut sprites = SpriteManager::default();
    let mut event_pump = window.event_pump()?;

    let tile_size = 16;
    let map_texture = textures.create_png_texture("assets/dungeon.png")?;
    let map_sprites = MapSprites::from_dungeon_spritesheet(map_texture, &mut sprites, tile_size);

    let GenGame {key, levels, player_start} = game_generator(tile_size, &map_sprites).generate(|| {
        let mut world = World::new();

        world.add_resource(FramesElapsed(1));
        world.add_resource(EventQueue::default());
        world.add_resource(ActionQueue::default());

        let mut dispatcher = DispatcherBuilder::new()
            .with(systems::Keyboard::default(), "Keyboard", &[])
            .with(systems::AI, "AI", &[])
            .with(systems::Physics, "Physics", &["Keyboard", "AI"])
            .with(systems::Interactions, "Interactions", &["Physics"])
            .with(systems::Animator, "Animator", &["Interactions"])
            .build();

        dispatcher.setup(&mut world.res);
        // Renderer is not called in the dispatcher, so we need to separately set up the component
        // storages for anything it uses.
        ui::setup(&mut world.res);

        (dispatcher, world)
    });

    // Add the character
    {
        let first_level = levels.first().as_mut()
            .expect("bug: should be at least one level")
            .world_mut();
        let character_texture = textures.create_png_texture("assets/hero.png")?;
        let character_animations = AnimationManager::standard_character_animations(fps as usize, character_texture, &mut sprites);
        first_level.create_entity()
            .with(KeyboardControlled)
            .with(CameraFocus)
            .with(Player)
            .with(HealthPoints(20))
            .with(Position(player_start))
            .with(BoundingBox::BottomHalf {width: 16, height: 8})
            .with(Movement::default())
            .with(Sprite(character_animations.default_sprite()))
            .with(character_animations.default_animation())
            .with(character_animations)
            .build();
    }

    let mut game_screen = GameScreen::new(player_start, levels);

    for (i, level) in game_screen.levels().enumerate() {
        level.render_to_file(format!("level{}.png", i+1))?;
    }

    let mut timer = window.timer()?;

    // Frames elapsed since the last render
    let mut last_frames_elapsed = 0;
    // Events since the last dispatch
    let mut events = Vec::new();
    let mut running = true;
    while running {
        let ticks = timer.ticks(); // ms

        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} | SDLEvent::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    running = false;
                },
                SDLEvent::KeyDown {scancode: Some(scancode), repeat: false, ..} => {
                    if let Some(scancode) = Key::from_scancode(scancode) {
                        events.push(Event::KeyDown(scancode));
                    }
                },
                SDLEvent::KeyUp {scancode: Some(scancode), repeat: false, ..} => {
                    if let Some(scancode) = Key::from_scancode(scancode) {
                        events.push(Event::KeyUp(scancode));
                    }
                },
                _ => {},
            }
        }

        let frames_elapsed = (ticks as f64 / 1000.0 * fps) as usize;
        let frames_elapsed_delta = frames_elapsed - last_frames_elapsed;

        // At least one frame must have passed for us to do anything
        if frames_elapsed_delta >= 1 {
            game_screen.dispatch(FramesElapsed(frames_elapsed_delta), events.drain(..).collect());
            game_screen.render(window.canvas_mut(), &textures, &sprites, &map_sprites)?;
            last_frames_elapsed = frames_elapsed;
        } else {
            let ms_per_frame = (1000.0 / fps) as u64;
            let ms_elapsed = (timer.ticks() - ticks) as u64;
            thread::sleep(Duration::from_millis(ms_per_frame - ms_elapsed));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;
    use rand::random;

    #[test]
    #[ignore] //TODO: Map generation still fails way too often
    fn map_generation() {
        // See if we can generate lots of maps without failing
        (0..500).into_par_iter().for_each(|_| {
            game_generator(16, unimplemented!()).generate(Default::default);
        });
    }

    #[test]
    fn deterministic_maps() {
        (0..10).into_par_iter().for_each(|_| {
            // The same key should produce the same map over and over again
            let key = random();
            let map1 = game_generator(16, unimplemented!()).generate_with_key(key);
            let map2 = game_generator(16, unimplemented!()).generate_with_key(key);
            assert_eq!(map1, map2);
        });
    }
}
