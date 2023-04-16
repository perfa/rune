use std::f32::consts;

use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::WindowCanvas,
};

use crate::{
    interface::{Interface, Player},
    level::{self, Child, Level, Node},
    wad::*,
};

pub struct Renderer {
    x_offset: i16,
    y_offset: i16,
    level_width: i16,
    level_height: i16,
    x_multiplier: f32,
    y_multiplier: f32,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            x_offset: 0,
            y_offset: 0,
            level_width: Interface::WIDTH as i16,
            level_height: Interface::HEIGHT as i16,
            x_multiplier: 1.0,
            y_multiplier: 1.0,
        }
    }

    pub fn find_bounds(&mut self, level: &Level) {
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
        self.x_multiplier = ((Interface::WIDTH - 12) * Interface::MULTIPLIER) as f32
            / f32::from(self.level_width)
            * 1000.;
        self.y_multiplier = ((Interface::HEIGHT - 12) * Interface::MULTIPLIER) as f32
            / self.level_height as f32
            * 1000.;
    }

    pub fn draw(
        &mut self,
        bsp_render: Option<u32>,
        player: &Player,
        level: &Level,
        canvas: &mut WindowCanvas,
    ) {
        self.draw_grid(canvas);
        self._draw_lines(&level, canvas);
        self.draw_verts(&level, canvas);
        self.draw_player(&player, canvas);
        self.draw_node(&player, &level, canvas);
        if let Some(bsp_render_depth) = bsp_render {
            self.draw_bsp_search(bsp_render_depth, &player, &level, canvas)
        }
    }

    fn adjust_coord(&self, x: i16, y: i16) -> (i32, i32) {
        let drawn_x = (x - self.x_offset) as i32 * self.x_multiplier.floor() as i32 / 1000;
        let drawn_y = (y - self.y_offset) as i32 * self.y_multiplier.floor() as i32 / 1000;
        (
            12 + drawn_x as i32 + Interface::MULTIPLIER as i32,
            ((Interface::HEIGHT - 12) * Interface::MULTIPLIER) as i32 - drawn_y as i32
                + Interface::MULTIPLIER as i32,
        )
    }

    fn adjust_dimension(&self, x: i16, y: i16) -> (u32, u32) {
        let drawn_x = x as i32 * self.x_multiplier.floor() as i32 / 1000;
        let drawn_y = y as i32 * self.y_multiplier.floor() as i32 / 1000;
        (drawn_x as u32, drawn_y as u32)
    }

    fn bbox_to_rect(&self, bbox: &BBox) -> Rect {
        let (x, y, w, h) = (bbox.left, bbox.top, bbox.width, bbox.height);
        let (drawn_x, drawn_y) = self.adjust_coord(x, y);
        let (drawn_w, drawn_h) = self.adjust_dimension(w, h);
        Rect::new(drawn_x, drawn_y, drawn_w, drawn_h)
    }

    fn angle_to_vertex(player: &Player, v: &Vertex) -> f32 {
        let dx = v.x as f32 - player.x;
        let dy = v.y as f32 - player.y;
        let angle = dy.atan2(dx);

        if angle < 0. {
            return angle + (2. * consts::PI);
        }

        angle
    }

    fn find_intersection(player: &Player, angle: f32, v1: &Vertex, v2: &Vertex) -> Option<Vertex> {
        let (x_min, y_min, x_max, y_max) = if v1.x < v2.x {
            (v1.x as f32, v1.y as f32, v2.x as f32, v2.y as f32)
        } else {
            (v2.x as f32, v2.y as f32, v1.x as f32, v1.y as f32)
        };

        // y = mx + b form for trace from player position
        let m1 = angle.tan();
        let b1 = (player.y as f32) - m1 * (player.x as f32);

        if v1.x == v2.x {
            // Seg is sticking straight up the Y axis
            let (x_intersect, y_intersect) = (v1.x as f32, m1 * v1.x as f32 + b1);
            return Some(Vertex {
                x: x_intersect.trunc() as i16,
                y: y_intersect.trunc() as i16,
            });
        }

        // y = mx + b form for Seg
        let m2 = (y_max - y_min) / (x_max - x_min);
        let b2 = y_min - m2 * x_min;
        //println!("{}x + {};  {}x + {}", m1, b1, m2, b2);

        // Check if the lines are parallel
        if m1 == m2 {
            // Technically the whole Seg is in view,
            // but it will render as 0 pixels, so skip it.
            return None;
        }

        let x_intersect = if m1 == f32::INFINITY || m1 == f32::NEG_INFINITY {
            // Angle-to-vertex is stick straight up the Y axis
            v1.x as f32
        } else {
            (b2 - b1) / (m1 - m2)
        };
        let y_intersect = m1 * x_intersect + b1;

        Some(Vertex {
            x: x_intersect.trunc() as i16,
            y: y_intersect.trunc() as i16,
        })
    }

    fn is_seg_visible(player: &Player, v1: &mut Vertex, v2: &mut Vertex) -> Option<(f32, f32)> {
        let mut a1 = Self::angle_to_vertex(player, v1);
        let mut a2 = Self::angle_to_vertex(player, v2);

        if a2 > a1 {
            return None;
        }

        let angle_diff = a1 - a2;
        if angle_diff >= consts::PI {
            return None;
        }

        // "Player FOV" is now 0°-90° (0-π/2) and tests will be dead simple
        let mut rotated_a1 = a1 - player.angle + consts::FRAC_PI_4;
        if rotated_a1 < 0. {
            rotated_a1 += consts::PI * 2.;
        } else if rotated_a1 > consts::PI * 2. {
            rotated_a1 -= consts::PI * 2.;
        }

        if rotated_a1 > consts::FRAC_PI_2 {
            rotated_a1 -= consts::FRAC_PI_2;
            // a1 further left than distance a1-a2
            // both points are left of view
            if rotated_a1 >= angle_diff {
                return None;
            }
            // To be used for clipping later
            a1 = player.angle + consts::FRAC_PI_4;
        }

        let mut rotated_a2 = a2 - player.angle;
        if rotated_a2 < 0. {
            rotated_a2 += consts::PI * 2.;
        } else if rotated_a2 > consts::PI * 2. {
            rotated_a2 -= consts::PI * 2.;
        }

        rotated_a2 = consts::FRAC_PI_4 - rotated_a2; // distance from left edge to our angle
                                                     // if greater than FOV
        if rotated_a2 < 0. {
            rotated_a2 += consts::PI * 2.;
        } else if rotated_a2 > consts::PI * 2. {
            rotated_a2 -= consts::PI * 2.;
        }
        if rotated_a2 > consts::FRAC_PI_2 {
            // To be used for clipping later
            a2 = player.angle - consts::FRAC_PI_4;
        }
        Some((a1, a2))
    }

    fn draw_player(&self, player: &Player, canvas: &mut WindowCanvas) {
        // println!("{} ({})", player.angle, player.angle.to_degrees());
        let (x, y) = self.adjust_coord(player.x.trunc() as i16, player.y.trunc() as i16);
        let (x1, y1) = (x - 2, y - 2);
        canvas.set_draw_color(Color::GREEN);
        canvas.draw_rect(Rect::new(x1, y1, 4, 4)).unwrap();

        let (view_x1, view_y1) = self.adjust_coord(
            ((player.angle.cos() * 2.) + player.x).trunc() as i16,
            ((player.angle.sin() * 2.) + player.y).trunc() as i16,
        );
        let (view_x2, view_y2) = self.adjust_coord(
            ((player.angle.cos() * 7.) + player.x).trunc() as i16,
            ((player.angle.sin() * 7.) + player.y).trunc() as i16,
        );
        canvas
            .draw_line(Point::new(view_x1, view_y1), Point::new(view_x2, view_y2))
            .unwrap();

        canvas.set_draw_color(Color::CYAN);
        let (left_los_x, left_los_y) = self.adjust_coord(
            ((f32::cos(player.angle + consts::FRAC_PI_4) * 4000.) + player.x).trunc() as i16,
            ((f32::sin(player.angle + consts::FRAC_PI_4) * 4000.) + player.y).trunc() as i16,
        );
        let (right_los_x, right_los_y) = self.adjust_coord(
            ((f32::cos(player.angle - consts::FRAC_PI_4) * 4000.) + player.x).trunc() as i16,
            ((f32::sin(player.angle - consts::FRAC_PI_4) * 4000.) + player.y).trunc() as i16,
        );

        canvas
            .draw_line(Point::new(x, y), Point::new(left_los_x, left_los_y))
            .unwrap();
        canvas
            .draw_line(Point::new(x, y), Point::new(right_los_x, right_los_y))
            .unwrap();
    }

    fn draw_verts(&self, level: &Level, canvas: &mut WindowCanvas) {
        canvas.set_draw_color(Color::CYAN);
        level.vertexes.iter().for_each(|Vertex { x, y }| {
            let (drawn_x, drawn_y) = self.adjust_coord(*x, *y);

            canvas.draw_point(Point::new(drawn_x, drawn_y)).unwrap();
        });
    }

    fn _draw_lines(&self, level: &Level, canvas: &mut WindowCanvas) {
        canvas.set_draw_color(Color::RED);
        level.linedefs.iter().for_each(|ld| {
            let v1 = level.vertexes[ld.start_vert];
            let v2 = level.vertexes[ld.end_vert];
            let (drawn_x1, drawn_y1) = self.adjust_coord(v1.x, v1.y);
            let (drawn_x2, drawn_y2) = self.adjust_coord(v2.x, v2.y);

            canvas
                .draw_line(
                    Point::new(drawn_x1, drawn_y1),
                    Point::new(drawn_x2, drawn_y2),
                )
                .unwrap();
        });
    }

    fn draw_sector(
        &self,
        ssec: &level::SubSector,
        level: &Level,
        player: &Player,
        canvas: &mut WindowCanvas,
    ) {
        ssec.segments.iter().for_each(|seg| {
            let mut v1 = level.vertexes[seg.start_vert].clone();
            let mut v2 = level.vertexes[seg.end_vert].clone();
            if let Some((a1, a2)) = Self::is_seg_visible(player, &mut v1, &mut v2) {
                let new_v1 = Renderer::find_intersection(player, a1, &v1, &v2).unwrap_or(v1);
                let new_v2 = Renderer::find_intersection(player, a2, &v1, &v2).unwrap_or(v2);
                let (drawn_x1, drawn_y1) = self.adjust_coord(new_v1.x, new_v1.y);
                let (drawn_x2, drawn_y2) = self.adjust_coord(new_v2.x, new_v2.y);

                canvas
                    .draw_line(
                        Point::new(drawn_x1, drawn_y1),
                        Point::new(drawn_x2, drawn_y2),
                    )
                    .unwrap();
            }
        })
    }
    fn draw_bsp(&self, node: &Node, level: &Level, player: &Player, canvas: &mut WindowCanvas) {
        match &node.left_child {
            Some(Child::NODE(n)) => self.draw_bsp(&n, level, player, canvas),
            Some(Child::SUBSECTOR(ssec)) => self.draw_sector(ssec, level, player, canvas),
            None => (),
        }
        match &node.right_child {
            Some(Child::NODE(n)) => self.draw_bsp(&n, level, player, canvas),
            Some(Child::SUBSECTOR(ssec)) => self.draw_sector(ssec, level, player, canvas),
            None => (),
        }
    }

    fn draw_node(&self, player: &Player, level: &Level, canvas: &mut WindowCanvas) {
        canvas.set_draw_color(Color::YELLOW);
        self.draw_bsp(&level.root_node, level, player, canvas);
        // if let Some(Child::NODE(n)) = &level.nodes.left_child {
        //     let (x1, y1) = self.adjust_coord(&n.left_bbox.left, &n.left_bbox.top);
        //     let (w1, h1) = self.adjust_dim(&n.left_bbox.width, &n.left_bbox.height);
        //     canvas.set_draw_color(Color::RED);
        //     canvas
        //         .draw_rect(Rect::new(x1, y1, w1 as u32, h1 as u32))
        //         .unwrap();

        //     let (x2, y2) = self.adjust_coord(&n.right_bbox.left, &n.right_bbox.top);
        //     let (w2, h2) = self.adjust_dim(&n.right_bbox.width, &n.right_bbox.height);
        //     canvas.set_draw_color(Color::GREEN);
        //     canvas
        //         .draw_rect(Rect::new(x2, y2, w2 as u32, h2 as u32))
        //         .unwrap();
        // }
    }

    fn draw_bsp_search(
        &self,
        depth: u32,
        player: &Player,
        level: &Level,
        canvas: &mut WindowCanvas,
    ) {
        if depth == 0 {
            canvas.set_draw_color(Color::CYAN);
            canvas
                .draw_rect(self.bbox_to_rect(&level.root_node.left_bbox))
                .unwrap();
            canvas.set_draw_color(Color::GREEN);
            canvas
                .draw_rect(self.bbox_to_rect(&level.root_node.right_bbox))
                .unwrap();
            return;
        }
        let child =
            level
                .root_node
                .find_partial(player.x.trunc() as i16, player.y.trunc() as i16, depth);
        match child {
            Child::NODE(n) => {
                canvas.set_draw_color(Color::CYAN);
                canvas.draw_rect(self.bbox_to_rect(&n.left_bbox)).unwrap();
                canvas.set_draw_color(Color::GREEN);
                canvas.draw_rect(self.bbox_to_rect(&n.right_bbox)).unwrap();
            }
            Child::SUBSECTOR(s) => self.draw_sector(s, level, player, canvas),
        }
    }

    fn draw_grid(&self, canvas: &mut WindowCanvas) {
        const GRID_SPACING: i16 = 128;
        let d_start_x = self.x_offset.rem_euclid(GRID_SPACING);
        let d_start_y = self.x_offset.rem_euclid(GRID_SPACING);

        let mut x = if self.x_offset < 0 {
            self.x_offset + d_start_x
        } else {
            self.x_offset - d_start_x
        };
        let origin_x = x;
        let mut y = if self.y_offset < 0 {
            self.y_offset + d_start_y
        } else {
            self.y_offset - d_start_y
        };
        while y < self.y_offset + self.level_height {
            while x < self.x_offset + self.level_width {
                let (x1, y1) = self.adjust_coord(x, y);
                canvas.set_draw_color(Color::WHITE);
                canvas.draw_point(Point::new(x1, y1)).unwrap();
                x += 128;
            }
            x = origin_x;
            y += 128;
        }
    }
}
