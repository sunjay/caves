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

mod systems;
mod components;
mod renderer;
mod resources;
mod texture_manager;
mod map;

use std::{
    thread,
    time::Duration,
};

use sdl2::{
    event::Event,
    keyboard::Keycode,
};
use specs::{
    Builder,
    DispatcherBuilder,
    World,
};

use components::{
    Position,
    Movement,
    BoundingBox,
    KeyboardControlled,
    CameraFocus,
    Sprite,
    AnimationManager,
};
use resources::{FramesElapsed, ActionQueue, GameKeys};
use texture_manager::TextureManager;
use renderer::Renderer;
use map::MapGenerator;

fn main() -> Result<(), String> {
    let fps = 60.0;

    let mut renderer = Renderer::init(320, 240)?;
    let texture_creator = renderer.texture_creator();
    let mut textures = TextureManager::new(&texture_creator);
    let mut event_pump = renderer.event_pump()?;

    let map = MapGenerator {
        texture_id: textures.create_png_texture("assets/dungeon.png")?,
        attempts: 1000,
        levels: 10,
        rows: 60,
        cols: 60,
        tile_size: 16,
        rooms: 6,
        room_rows: (10, 18).into(),
        room_cols: (8, 14).into(),
        passage_size: 4,
        treasure_chamber_width: 7,
        treasure_chamber_height: 9,
        doors: 2,
        next_prev_tiles: 2,
    }.generate();

    let mut world = World::new();

    world.add_resource(map.clone());
    world.add_resource(FramesElapsed(1));
    world.add_resource(ActionQueue::default());
    world.add_resource(GameKeys::from(event_pump.keyboard_state()));

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::Keyboard, "Keyboard", &[])
        .with(systems::Physics, "Physics", &["Keyboard"])
        .with(systems::Animator, "Animator", &["Physics"])
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
        .with(Position(character_center))
        .with(BoundingBox {width: 32, height: 30})
        .with(Movement::default())
        .with(Sprite(character_animations.default_sprite()))
        .with(character_animations.default_animation())
        .with(character_animations)
        .build();

    let mut timer = renderer.timer()?;

    // Frames elapsed since the last render
    let mut last_frames_elapsed = 0;
    let mut running = true;
    while running {
        let ticks = timer.ticks(); // ms

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    running = false;
                },
                _ => {},
            }
        }

        let frames_elapsed = (ticks as f64 / 1000.0 * fps) as usize;
        let frames_elapsed_delta = frames_elapsed - last_frames_elapsed;

        // At least one frame must have passed for us to do anything
        if frames_elapsed_delta >= 1 {
            *world.write_resource::<FramesElapsed>() = FramesElapsed(frames_elapsed_delta);
            *world.write_resource::<GameKeys>() = GameKeys::from(event_pump.keyboard_state());
            *world.write_resource::<ActionQueue>() = ActionQueue::default();

            dispatcher.dispatch(&mut world.res);

            renderer.render(&world, &textures)?;
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
