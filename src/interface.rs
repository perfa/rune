use std::{
    cmp,
    time::{Duration, Instant},
};

use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Point, render::WindowCanvas};

use crate::wad::{LevelData, Linedef, Thing, Vertex, WadFile};

pub struct Interface {
    x_offset: i16,
    y_offset: i16,
    level_width: i16,
    level_height: i16,
    x_multiplier: f64,
    y_multiplier: f64,
}

impl Interface {
    pub const WIDTH: u32 = 320;
    pub const HEIGHT: u32 = 240;
    pub const MULTIPLIER: u32 = 4;

    pub fn new() -> Self {
        Interface {
            x_offset: 0,
            y_offset: 0,
            level_width: Self::WIDTH as i16,
            level_height: Self::HEIGHT as i16,
            x_multiplier: 1.0,
            y_multiplier: 1.0,
        }
    }

    pub fn run(&mut self, wad: &WadFile) {
        let mut current_level = 0;
        let level_count = wad.levels.len();
        let mut level = &wad.levels[current_level];
        self.find_bounds(level);
        let player = level.things.iter().find(|t| t.thing_type == 1).unwrap();
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
                        self.find_bounds(level);
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
                        self.find_bounds(level);
                    }
                    _ => {}
                }
            }

            // DRAW SOMETHING
            self.draw_lines(level, &mut canvas);
            self.draw_verts(level, &mut canvas);
            self.draw_player(player, &mut canvas);

            canvas.present();
            let cycle_time = Instant::now() - loop_start;
            let one_sixieth_second = Duration::new(0, 1_000_000_000u32 / 60);
            if one_sixieth_second > cycle_time {
                let remaining = one_sixieth_second - cycle_time;
                ::std::thread::sleep(remaining);
            }
        }
    }

    fn find_bounds(&mut self, level: &LevelData) {
        let mut min_x = i16::MAX;
        let mut max_x = i16::MIN;
        let mut min_y = i16::MAX;
        let mut max_y = i16::MIN;
        for v in level.vertexes.iter() {
            if v.x < min_x {
                min_x = v.x;
            }
            if v.x > max_x {
                max_x = v.x;
            }
            if v.y < min_y {
                min_y = v.y;
            }
            if v.y > max_y {
                max_y = v.y;
            }
        }
        self.x_offset = min_x;
        self.y_offset = min_y;
        self.level_width = max_x - min_x;
        self.level_height = max_y - min_y;
        self.x_multiplier =
            f64::from((Self::WIDTH - 2) * Self::MULTIPLIER) / self.level_width as f64 * 1000.;
        self.y_multiplier =
            f64::from((Self::HEIGHT - 2) * Self::MULTIPLIER) / self.level_height as f64 * 1000.;
    }

    fn adjust_coord(&self, x: &i16, y: &i16) -> (i32, i32) {
        let drawn_x = (x - self.x_offset) as i32 * self.x_multiplier.floor() as i32 / 1000;
        let drawn_y = (y - self.y_offset) as i32 * self.y_multiplier.floor() as i32 / 1000;
        (
            drawn_x as i32 + Self::MULTIPLIER as i32,
            drawn_y as i32 + Self::MULTIPLIER as i32,
        )
    }

    fn draw_player(&self, player: &Thing, canvas: &mut WindowCanvas) {
        let (x, y) = self.adjust_coord(&player.x, &player.y);
        canvas.set_draw_color(Color::GREEN);
        canvas.draw_point(Point::new(x, y)).unwrap();
    }

    fn draw_verts(&self, level: &LevelData, canvas: &mut WindowCanvas) {
        canvas.set_draw_color(Color::YELLOW);
        level.vertexes.iter().for_each(|Vertex { x, y }| {
            let (drawn_x, drawn_y) = self.adjust_coord(x, y);

            canvas.draw_point(Point::new(drawn_x, drawn_y)).unwrap();
        });
    }

    fn draw_lines(&self, level: &LevelData, canvas: &mut WindowCanvas) {
        canvas.set_draw_color(Color::RED);
        level.linedefs.iter().for_each(
            |Linedef {
                 start_vert,
                 end_vert,
                 ..
             }| {
                let v1 = level.vertexes[*start_vert];
                let v2 = level.vertexes[*end_vert];
                let (drawn_x1, drawn_y1) = self.adjust_coord(&v1.x, &v1.y);
                let (drawn_x2, drawn_y2) = self.adjust_coord(&v2.x, &v2.y);

                canvas
                    .draw_line(
                        Point::new(drawn_x1, drawn_y1),
                        Point::new(drawn_x2, drawn_y2),
                    )
                    .unwrap();
            },
        );
    }
}
