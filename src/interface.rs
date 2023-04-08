use std::{
    cmp,
    f64::consts,
    time::{Duration, Instant},
};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    render::WindowCanvas,
};

use crate::wad::{LevelData, Linedef, Vertex, WadFile};

struct Player {
    x: i16,
    y: i16,
    angle: f64,
}

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
        let player_thing = level.things.iter().find(|t| t.thing_type == 1).unwrap();
        let mut player = Player {
            x: player_thing.x,
            y: player_thing.y,
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
                        player.x = player_thing.x;
                        player.y = player_thing.y;
                        player.angle = player_thing.angle_facing;
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
                        let player_thing = level.things.iter().find(|t| t.thing_type == 1).unwrap();
                        player.x = player_thing.x;
                        player.y = player_thing.y;
                        player.angle = player_thing.angle_facing;
                        self.find_bounds(level);
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => {
                        player.angle -= 0.34;
                        if player.angle < 0.0 {
                            player.angle += 2.0 * consts::PI;
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => {
                        player.angle += 0.34;
                        if player.angle > 2.0 * consts::PI {
                            player.angle -= 2.0 * consts::PI;
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Up),
                        ..
                    } => {
                        player.x += 3 * (f64::cos(player.angle + consts::PI) * 2.).floor() as i16;
                        player.y -= 3 * (f64::sin(player.angle + consts::PI) * 2.).floor() as i16;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => {
                        player.x -= 3 * (f64::cos(player.angle + consts::PI) * 2.).floor() as i16;
                        player.y += 3 * (f64::sin(player.angle + consts::PI) * 2.).floor() as i16;
                    }
                    _ => {}
                }
            }

            // DRAW SOMETHING
            self.draw_lines(level, &mut canvas);
            self.draw_verts(level, &mut canvas);
            self.draw_player(&player, &mut canvas);
            self.draw_node(level, &mut canvas);
            self.draw_sectors(&player, level, &mut canvas);

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
            f64::from((Self::WIDTH - 12) * Self::MULTIPLIER) / self.level_width as f64 * 1000.;
        self.y_multiplier =
            f64::from((Self::HEIGHT - 12) * Self::MULTIPLIER) / self.level_height as f64 * 1000.;
    }

    fn adjust_coord(&self, x: &i16, y: &i16) -> (i32, i32) {
        let drawn_x = (x - self.x_offset) as i32 * self.x_multiplier.floor() as i32 / 1000;
        let drawn_y = (y - self.y_offset) as i32 * self.y_multiplier.floor() as i32 / 1000;
        (
            12 + drawn_x as i32 + Self::MULTIPLIER as i32,
            ((Self::HEIGHT - 12) * Self::MULTIPLIER) as i32 - drawn_y as i32
                + Self::MULTIPLIER as i32,
        )
    }

    fn adjust_dim(&self, w: &i16, h: &i16) -> (i32, i32) {
        let drawn_w = *w as i32 * self.x_multiplier.floor() as i32 / 1000;
        let drawn_h = *h as i32 * self.y_multiplier.floor() as i32 / 1000;
        (
            drawn_w as i32 + Self::MULTIPLIER as i32,
            drawn_h as i32 + Self::MULTIPLIER as i32,
        )
    }

    fn draw_sectors(&self, player: &Player, level: &LevelData, canvas: &mut WindowCanvas) {
        let ssec = level.nodes.find(player.x, player.y);
        canvas.set_draw_color(Color::YELLOW);
        level.segs[ssec.first_segment..ssec.first_segment + ssec.segment_count]
            .into_iter()
            .for_each(|seg| {
                let v_s = level.vertexes.get(seg.start_vert).cloned().unwrap();
                let v_e = level.vertexes.get(seg.end_vert).cloned().unwrap();
                let (x1, y1) = self.adjust_coord(&v_s.x, &v_s.y);
                let (x2, y2) = self.adjust_coord(&v_e.x, &v_e.y);
                canvas
                    .draw_line(Point::new(x1, y1), Point::new(x2, y2))
                    .unwrap();
            });
        // canvas.set_draw_color(Color::YELLOW);

        // level.segs[s..e].iter().for_each(|seg| {
        //     let v1 = level.vertexes[seg.start_vert as usize];
        //     let v2 = level.vertexes[seg.end_vert as usize];
        //     let (x1, y1) = self.adjust_coord(&v1.x, &v1.y);
        //     let (x2, y2) = self.adjust_coord(&v2.x, &v2.y);
        //     canvas
        //         .draw_line(Point::new(x1, y1), Point::new(x2, y2))
        //         .unwrap();
        // });
    }

    fn draw_player(&self, player: &Player, canvas: &mut WindowCanvas) {
        let (x, y) = self.adjust_coord(&player.x, &player.y);
        let (x1, y1) = (x - 2, y - 2);
        canvas.set_draw_color(Color::GREEN);
        canvas.draw_rect(Rect::new(x1, y1, 4, 4)).unwrap();

        let view_x1 = (f64::cos(player.angle + consts::PI) * 2.).floor() as i32 + x;
        let view_y1 = (f64::sin(player.angle + consts::PI) * 2.).floor() as i32 + y;
        let view_x2 = (f64::cos(player.angle + consts::PI) * 7.).floor() as i32 + x;
        let view_y2 = (f64::sin(player.angle + consts::PI) * 7.).floor() as i32 + y;
        canvas
            .draw_line(Point::new(view_x1, view_y1), Point::new(view_x2, view_y2))
            .unwrap();
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

    fn draw_node(&self, level: &LevelData, canvas: &mut WindowCanvas) {
        let n = &level.nodes;
        let (x1, y1) = self.adjust_coord(&n.left_bbox.left, &n.left_bbox.top);
        let (w1, h1) = self.adjust_dim(&n.left_bbox.width, &n.left_bbox.height);
        canvas.set_draw_color(Color::RED);
        canvas
            .draw_rect(Rect::new(x1, y1, w1 as u32, h1 as u32))
            .unwrap();

        let (x2, y2) = self.adjust_coord(&n.right_bbox.left, &n.right_bbox.top);
        let (w2, h2) = self.adjust_dim(&n.right_bbox.width, &n.right_bbox.height);
        canvas.set_draw_color(Color::GREEN);
        canvas
            .draw_rect(Rect::new(x2, y2, w2 as u32, h2 as u32))
            .unwrap();

        let (x1, y1) = self.adjust_coord(&n.partition_x, &n.partition_y);
        //let (dx, dy) = self.adjust_dim(&n.delta_x, &n.delta_y);
        canvas.set_draw_color(Color::WHITE);
        canvas.draw_point(Point::new(x1, y1)).unwrap();
        // canvas.set_draw_color(Color::BLUE);
        // canvas
        //     .draw_line(Point::new(x1, y1), Point::new(x1 + dx, y1 + dy))
        //     .unwrap();
    }
}
