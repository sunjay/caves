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
mod renderer;
mod resources;
mod texture_manager;
mod map;
mod sprites;

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
    Enemy,
};
use resources::{FramesElapsed, ActionQueue, EventQueue, Event, Key};
use texture_manager::TextureManager;
use renderer::Renderer;
use map::{MapGenerator, GameMap, EnemyState};
use sprites::MapSprites;

fn map_generator(tile_size: u32) -> MapGenerator {
    MapGenerator {
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
    }
}

fn main() -> Result<(), String> {
    let fps = 30.0;

    let mut renderer = Renderer::init(320, 240)?;
    let texture_creator = renderer.texture_creator();
    let mut textures = TextureManager::new(&texture_creator);
    let mut event_pump = renderer.event_pump()?;

    let map_texture = textures.create_png_texture("assets/dungeon.png")?;
    let tile_size = 16;
    let sprites = MapSprites::from_dungeon_spritesheet(map_texture, tile_size);

    let map = map_generator(tile_size).generate();

    for (i, level) in map.levels().enumerate() {
        level.render_to_file(format!("level{}.png", i+1))?;
    }

    let mut world = World::new();

    world.add_resource(map.clone());
    world.add_resource(FramesElapsed(1));
    world.add_resource(EventQueue::default());
    world.add_resource(ActionQueue::default());

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::Keyboard::default(), "Keyboard", &[])
        .with(systems::Physics, "Physics", &["Keyboard"])
        .with(systems::Interactions, "Interactions", &["Physics"])
        .with(systems::Animator, "Animator", &["Interactions"])
        .build();
    dispatcher.setup(&mut world.res);
    // Renderer is not called in the dispatcher, so we need to separately set up the component
    // storages for anything it uses.
    Renderer::setup(&mut world.res);

    // Add the character
    let character_center = map.game_start();
    let character_texture = textures.create_png_texture("assets/hero.png")?;
    let character_animations = AnimationManager::standard_character_animations(fps as usize, character_texture);
    world.create_entity()
        .with(KeyboardControlled)
        .with(CameraFocus)
        .with(Player)
        .with(HealthPoints(20))
        .with(Position(character_center))
        .with(BoundingBox::BottomHalf {width: 16, height: 8})
        .with(Movement::default())
        .with(Sprite(character_animations.default_sprite()))
        .with(character_animations.default_animation())
        .with(character_animations)
        .build();

    let enemies: Vec<_> = {
        let mut map = world.write_resource::<GameMap>();
        let level = map.current_level_map_mut();
        //TODO: No need to create and return this variable with NLL
        let enemies = level.clear_enemies().collect();
        enemies
    };
    for enemy in enemies {
        // Explicitly pattern matching so that when this struct changes, rustc will tell us here
        let EnemyState {position, health, bounding_box, movement, sprite, animation, animation_manager} = enemy;
        world.create_entity()
            .with(Enemy)
            .with(position)
            .with(health)
            .with(bounding_box)
            .with(movement)
            .with(sprite)
            .with(animation)
            .with(animation_manager)
            .build();
    }

    let mut timer = renderer.timer()?;

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
                    Key::from_scancode(scancode)
                        .map(|scancode| events.push(Event::KeyDown(scancode)));
                },
                SDLEvent::KeyUp {scancode: Some(scancode), repeat: false, ..} => {
                    Key::from_scancode(scancode)
                        .map(|scancode| events.push(Event::KeyUp(scancode)));
                },
                _ => {},
            }
        }

        let frames_elapsed = (ticks as f64 / 1000.0 * fps) as usize;
        let frames_elapsed_delta = frames_elapsed - last_frames_elapsed;

        // At least one frame must have passed for us to do anything
        if frames_elapsed_delta >= 1 {
            *world.write_resource::<FramesElapsed>() = FramesElapsed(frames_elapsed_delta);
            *world.write_resource::<ActionQueue>() = ActionQueue::default();
            *world.write_resource::<EventQueue>() = EventQueue(events.drain(..).collect());

            dispatcher.dispatch(&mut world.res);

            renderer.render(&world, &textures, &sprites)?;
            last_frames_elapsed = frames_elapsed;

            // Register any updates
            world.maintain();
        }
        else {
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
            map_generator(16).generate();
        });
    }

    #[test]
    fn deterministic_maps() {
        (0..10).into_par_iter().for_each(|_| {
            // The same key should produce the same map over and over again
            let key = random();
            let map1 = map_generator(16).generate_with_key(key);
            let map2 = map_generator(16).generate_with_key(key);
            assert_eq!(map1, map2);
        });
    }
}
