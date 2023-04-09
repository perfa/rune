use std::{
    cmp,
    collections::HashSet,
    f32::consts,
    time::{Duration, Instant},
};

use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::Color,
    EventPump,
};

use crate::{renderer::Renderer, wad::WadFile};

enum GameState {
    Viewing,
    Playing,
    Paused,
    TitleScreen,
    GameOver,
}

#[derive(Copy, Clone, Debug)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
}

pub struct Interface {
    state: GameState,
    pressed_keys: HashSet<Scancode>,
}

impl Interface {
    pub const WIDTH: u32 = 320;
    pub const HEIGHT: u32 = 240;
    pub const MULTIPLIER: u32 = 4;

    pub fn new() -> Self {
        Interface {
            pressed_keys: HashSet::new(),
            state: GameState::Viewing,
        }
    }

    pub fn run(&mut self, wad: &WadFile) {
        let mut current_level = 0;
        let level_count = wad.levels.len();
        let mut level = &wad.levels[current_level];
        let mut renderer = Renderer::new();
        renderer.find_bounds(level);
        let player_thing = level.things.iter().find(|t| t.thing_type == 1).unwrap();
        let mut player = Player {
            x: f32::from(player_thing.x),
            y: f32::from(player_thing.y),
            angle: player_thing.angle_facing,
        };
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "Rune",
                Self::WIDTH * Self::MULTIPLIER,
                Self::HEIGHT * Self::MULTIPLIER,
            )
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();
        let mut event_pump = sdl_context.event_pump().unwrap();
        'running: loop {
            let loop_start = Instant::now();
            canvas.set_draw_color(Color::BLACK);
            canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Q),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Period),
                        ..
                    } => {
                        current_level = cmp::min(level_count - 1, current_level + 1);
                        level = &wad.levels[current_level];
                        let player_thing = level.things.iter().find(|t| t.thing_type == 1).unwrap();
                        player.x = f32::from(player_thing.x);
                        player.y = f32::from(player_thing.y);
                        player.angle = player_thing.angle_facing;
                        renderer.find_bounds(level);
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Comma),
                        ..
                    } => {
                        current_level = if current_level > 0 {
                            current_level - 1
                        } else {
                            0
                        };
                        level = &wad.levels[current_level];
                        let player_thing = level.things.iter().find(|t| t.thing_type == 1).unwrap();
                        player.x = f32::from(player_thing.x);
                        player.y = f32::from(player_thing.y);
                        player.angle = player_thing.angle_facing;
                        renderer.find_bounds(level);
                    }
                    _ => {}
                }
            }
            self.handle_input(&mut player, &mut event_pump);

            // DRAW SOMETHING
            renderer.draw(&player, level, &mut canvas);

            canvas.present();
            let cycle_time = Instant::now() - loop_start;
            let one_sixieth_second = Duration::new(0, 1_000_000_000u32 / 60);
            if one_sixieth_second > cycle_time {
                let remaining = one_sixieth_second - cycle_time;
                ::std::thread::sleep(remaining);
            }
        }
    }

    fn get_scancodes(old: &HashSet<Scancode>, new: &HashSet<Scancode>) -> HashSet<Scancode> {
        new - old
    }

    fn handle_input(&mut self, player: &mut Player, event_pump: &mut EventPump) {
        let scancodes: HashSet<Scancode> =
            event_pump.keyboard_state().pressed_scancodes().collect();

        let newly_pressed: HashSet<Scancode> =
            Interface::get_scancodes(&self.pressed_keys, &scancodes);
        self.pressed_keys = scancodes;

        match self.state {
            GameState::TitleScreen => {
                if newly_pressed.contains(&Scancode::Space) {
                    self.state = GameState::Playing
                }
            }
            GameState::Playing | GameState::Viewing => {
                if newly_pressed.contains(&Scancode::P) {
                    self.state = GameState::Paused;
                }

                if self.pressed_keys.contains(&Scancode::Up) {
                    player.x += f32::cos(player.angle) * 3.;
                    player.y += f32::sin(player.angle) * 3.;
                } else if self.pressed_keys.contains(&Scancode::Down) {
                    player.x -= f32::cos(player.angle) * 3.;
                    player.y -= f32::sin(player.angle) * 3.;
                }
                if self.pressed_keys.contains(&Scancode::Left) {
                    player.angle += 0.05;
                    if player.angle > 2.0 * consts::PI {
                        player.angle -= 2.0 * consts::PI;
                    }
                } else if self.pressed_keys.contains(&Scancode::Right) {
                    player.angle -= 0.05;
                    if player.angle < 0.0 {
                        player.angle += 2.0 * consts::PI;
                    }
                }
            }
            GameState::Paused => {
                if newly_pressed.contains(&Scancode::P) {
                    self.state = GameState::Playing;
                }
            }
            GameState::GameOver => {
                if newly_pressed.contains(&Scancode::Space) {
                    // TODO: Reset
                    self.state = GameState::Playing
                }
            }
        }
    }
}
