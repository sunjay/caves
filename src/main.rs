#![deny(unused_must_use)]

#[macro_use]
extern crate specs_derive;
#[macro_use]
extern crate shred_derive;
#[macro_use]
extern crate lazy_static;

use sdl2;
use shred;

mod systems;
mod components;
mod generator;
mod resources;
mod map;
mod ui;
mod map_sprites;
mod assets;

use std::{thread,time::Duration};

use sdl2::{event::Event as SDLEvent, keyboard::Keycode};
use specs::{DispatcherBuilder, World};

use crate::components::{
    PlayerComponents,
    Position,
    HealthPoints,
    Movement,
    BoundingBox,
    KeyboardControlled,
    CameraFocus,
    Sprite,
    Player,
};
use crate::assets::AssetManager;
use crate::resources::{FramesElapsed, ChangeGameState, ActionQueue, EventQueue, Event, Key};
use crate::ui::{Window, GameScreen, SDLError, RenderContext};
use crate::generator::{GameGenerator, GenGame};
use crate::map_sprites::MapSprites;

fn game_generator(tile_size: u32, map_sprites: &MapSprites) -> GameGenerator<'_> {
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
    let mut event_pump = window.event_pump()?;

    let tile_size = 16;
    let AssetManager {
        textures,
        map_sprites,
        player_animations,
        sprites,
    } = AssetManager::load(&texture_creator, fps as usize, tile_size)?;

    let keyboard_system = systems::Keyboard::default();
    let GenGame {key, levels, player_start} = game_generator(tile_size, &map_sprites).generate(|| {
        let mut world = World::new();

        world.add_resource(FramesElapsed(1));
        world.add_resource(ChangeGameState::default());
        world.add_resource(EventQueue::default());
        world.add_resource(ActionQueue::default());

        let mut dispatcher = DispatcherBuilder::new()
            .with(keyboard_system.clone(), "Keyboard", &[])
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

    println!("Map Key: {}", key);

    // Add the character
    let player = PlayerComponents {
        keyboard_controlled: KeyboardControlled,
        camera_focus: CameraFocus,
        player: Player,
        health_points: HealthPoints(20),
        position: Position(player_start),
        bounding_box: BoundingBox::BottomHalf {width: 16, height: 8},
        movement: Movement::default(),
        sprite: Sprite(player_animations.default_sprite()),
        animation: player_animations.default_animation(),
        animation_manager: player_animations,
    };

    let mut game_screen = GameScreen::new(player, levels);

    for (i, level) in game_screen.levels().enumerate() {
        level.render_to_file(format!("level{}.png", i+1))?;
    }

    let mut timer = window.timer()?;
    let mut ctx = RenderContext::new(window.canvas_mut(), &textures, &sprites, &map_sprites);

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

            ctx.canvas.clear();
            game_screen.render(&mut ctx)?;
            ctx.canvas.present();

            last_frames_elapsed = frames_elapsed;
        } else {
            let ms_per_frame = (1000.0 / fps) as u64;
            let ms_elapsed = (timer.ticks() - ticks) as u64;
            thread::sleep(Duration::from_millis(ms_per_frame - ms_elapsed));
        }
    }

    Ok(())
}
