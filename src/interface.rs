use std::time::{Duration, Instant};

use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Point, render::WindowCanvas};

use crate::wad::{LevelData, Linedef, Vertex};

pub struct Interface {}

impl Interface {
    pub const WIDTH: u32 = 320;
    pub const HEIGHT: u32 = 240;
    pub const MULTIPLIER: u32 = 4;

    pub fn new() -> Self {
        Interface {}
    }

    pub fn run(&mut self, level: &LevelData) {
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
                    _ => {}
                }
            }

            // DRAW SOMETHING
            self.draw_lines(level, &mut canvas);
            self.draw_verts(level, &mut canvas);

            canvas.present();
            let cycle_time = Instant::now() - loop_start;
            let one_sixieth_second = Duration::new(0, 1_000_000_000u32 / 60);
            if one_sixieth_second > cycle_time {
                let remaining = one_sixieth_second - cycle_time;
                ::std::thread::sleep(remaining);
            }
        }
    }

    fn find_bounds(&self, level: &LevelData) -> (i16, i16, i16, i16) {
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
        let x_offset = min_x;
        let y_offset = min_y;
        let level_width = max_x - min_x;
        let level_height = max_y - min_y;
        (x_offset, y_offset, level_width, level_height)
    }

    fn draw_verts(&self, level: &LevelData, canvas: &mut WindowCanvas) {
        let (x_offset, y_offset, level_width, level_height) = self.find_bounds(level);
        let x_multiplier =
            f64::from((Self::WIDTH - 2) * Self::MULTIPLIER) / level_width as f64 * 1000.;
        let y_multiplier =
            f64::from((Self::HEIGHT - 2) * Self::MULTIPLIER) / level_height as f64 * 1000.;

        canvas.set_draw_color(Color::YELLOW);
        for Vertex { x, y } in level.vertexes.iter() {
            let drawn_x = (x - x_offset) as i32 * x_multiplier.floor() as i32 / 1000;
            let drawn_y = (y - y_offset) as i32 * y_multiplier.floor() as i32 / 1000;

            canvas
                .draw_point(Point::new(
                    drawn_x as i32 + Self::MULTIPLIER as i32,
                    drawn_y as i32 + Self::MULTIPLIER as i32,
                ))
                .unwrap();
        }
    }

    fn draw_lines(&self, level: &LevelData, canvas: &mut WindowCanvas) {
        let (x_offset, y_offset, level_width, level_height) = self.find_bounds(level);
        let x_multiplier =
            f64::from((Self::WIDTH - 2) * Self::MULTIPLIER) / level_width as f64 * 1000.;
        let y_multiplier =
            f64::from((Self::HEIGHT - 2) * Self::MULTIPLIER) / level_height as f64 * 1000.;

        canvas.set_draw_color(Color::RED);
        level.linedefs.iter().for_each(
            |Linedef {
                 start_vert,
                 end_vert,
                 ..
             }| {
                let v1 = level.vertexes[*start_vert];
                let v2 = level.vertexes[*end_vert];

                let drawn_x1 = (v1.x - x_offset) as i32 * x_multiplier.floor() as i32 / 1000;
                let drawn_y1 = (v1.y - y_offset) as i32 * y_multiplier.floor() as i32 / 1000;
                let drawn_x2 = (v2.x - x_offset) as i32 * x_multiplier.floor() as i32 / 1000;
                let drawn_y2 = (v2.y - y_offset) as i32 * y_multiplier.floor() as i32 / 1000;

                canvas
                    .draw_line(
                        Point::new(
                            drawn_x1 as i32 + Self::MULTIPLIER as i32,
                            drawn_y1 as i32 + Self::MULTIPLIER as i32,
                        ),
                        Point::new(
                            drawn_x2 as i32 + Self::MULTIPLIER as i32,
                            drawn_y2 as i32 + Self::MULTIPLIER as i32,
                        ),
                    )
                    .unwrap();
            },
        );
    }
}
