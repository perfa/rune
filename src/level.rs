// Sector > Sidedef > Linedef > Segment > Subsector >> BSP

use std::{collections::HashMap, rc::Rc};

use crate::wad::{BBox, ChildIdx, LevelData, Sector, Thing, Vertex};

#[derive(Clone, Debug)]
pub struct Sidedef {
    pub x_off: i16,
    pub y_off: i16,
    pub upper_tex: String,
    pub lower_tex: String,
    pub middle_tex: String,
    pub sector: Rc<Sector>,
}

impl Sidedef {
    pub fn new(
        x_off: i16,
        y_off: i16,
        upper_tex: String,
        lower_tex: String,
        middle_tex: String,
        sector: Rc<Sector>,
    ) -> Self {
        Sidedef {
            x_off,
            y_off,
            upper_tex,
            lower_tex,
            middle_tex,
            sector,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Linedef {
    pub start_vert: usize,
    pub end_vert: usize,
    pub flags: i16,
    pub special_type: i16,
    pub sector_tag: usize,
    pub right_sidedef: Option<Rc<Sidedef>>,
    pub left_sidedef: Option<Rc<Sidedef>>,
}

impl Linedef {
    pub fn new(
        start_vert: usize,
        end_vert: usize,
        flags: i16,
        special_type: i16,
        sector_tag: usize,
        right_sidedef: Option<Rc<Sidedef>>,
        left_sidedef: Option<Rc<Sidedef>>,
    ) -> Self {
        Linedef {
            start_vert,
            end_vert,
            flags,
            special_type,
            sector_tag,
            right_sidedef,
            left_sidedef,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub start_vert: usize,
    pub end_vert: usize,
    pub angle: f32,
    pub linedef: Rc<Linedef>,
    pub dir_like_linedef: bool,
    pub offset: i16,
}
impl Segment {
    pub fn new(
        start_vert: usize,
        end_vert: usize,
        angle: f32,
        linedef: Rc<Linedef>,
        dir_like_linedef: bool,
        offset: i16,
    ) -> Self {
        Segment {
            start_vert,
            end_vert,
            angle,
            linedef,
            dir_like_linedef,
            offset,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubSector {
    pub segment_count: usize,
    pub segments: Vec<Rc<Segment>>,
}

#[derive(Debug)]
pub enum Child {
    NODE(Rc<Node>),
    SUBSECTOR(Rc<SubSector>),
}

#[derive(Debug)]
pub struct Node {
    pub partition_x: i16,
    pub partition_y: i16,
    pub delta_x: i16,
    pub delta_y: i16,
    pub right_bbox: BBox,
    pub left_bbox: BBox,
    pub right_child: Option<Child>,
    pub left_child: Option<Child>,
}

impl Node {
    pub fn find_partial(&self, x: i16, y: i16, levels: u32) -> &Child {
        let child = if !self.is_point_behind(x, y) {
            self.left_child.as_ref().unwrap()
        } else {
            self.right_child.as_ref().unwrap()
        };
        if levels == 1 {
            return child;
        }
        match child {
            Child::NODE(n) => n.find_partial(x, y, levels - 1),
            Child::SUBSECTOR(_) => child,
        }
    }
    pub fn _find(&self, x: i16, y: i16) -> &SubSector {
        let child = if self.is_point_behind(x, y) {
            self.left_child.as_ref().unwrap()
        } else {
            self.right_child.as_ref().unwrap()
        };
        match child {
            Child::NODE(n) => n._find(x, y),
            Child::SUBSECTOR(s) => &s,
        }
    }

    pub fn is_point_behind(&self, x: i16, y: i16) -> bool {
        if self.delta_x == 0 {
            if x <= self.partition_x {
                return self.delta_y > 0;
            } else {
                return self.delta_y < 0;
            }
        }
        if self.delta_y == 0 {
            if y <= self.partition_y {
                return self.delta_x < 0;
            } else {
                return self.delta_x > 0;
            }
        }

        let x2 = self.partition_x + self.delta_x;
        let y2 = self.partition_y + self.delta_y;
        return (x2 - self.partition_x) as i32 * (y - self.partition_y) as i32
            > (y2 - self.partition_y) as i32 * (x - self.partition_x) as i32;
    }
}

pub struct Level {
    pub vertexes: Vec<Vertex>,
    pub things: Vec<Thing>,
    pub sectors: Vec<Rc<Sector>>,
    pub sidedefs: Vec<Rc<Sidedef>>,
    pub linedefs: Vec<Rc<Linedef>>,
    pub segments: Vec<Rc<Segment>>,
    pub subsectors: Vec<Rc<SubSector>>,
    pub nodes: HashMap<i16, Rc<Node>>,
    pub root_node: Rc<Node>,
}

impl Level {
    pub fn new(data: &LevelData) -> Self {
        let sectors: Vec<Rc<Sector>> = data
            .sectors
            .iter()
            .map(|data| Rc::new(data.clone()))
            .collect();

        let sidedefs: Vec<Rc<Sidedef>> = data
            .sidedefs
            .iter()
            .map(|data| {
                Rc::new(Sidedef::new(
                    data.x_off,
                    data.y_off,
                    data.upper_tex.clone(),
                    data.lower_tex.clone(),
                    data.middle_tex.clone(),
                    Rc::clone(&sectors[data.sector]),
                ))
            })
            .collect();

        let linedefs: Vec<Rc<Linedef>> = data
            .linedefs
            .iter()
            .map(|data| {
                Rc::new(Linedef::new(
                    data.start_vert,
                    data.end_vert,
                    data.flags,
                    data.special_type,
                    data.sector_tag,
                    if data.right_sidedef >= 65535 {
                        None
                    } else {
                        Some(Rc::clone(&sidedefs[data.right_sidedef]))
                    },
                    if data.left_sidedef >= 65535 {
                        None
                    } else {
                        Some(Rc::clone(&sidedefs[data.left_sidedef]))
                    },
                ))
            })
            .collect();

        let segments: Vec<Rc<Segment>> = data
            .segs
            .iter()
            .map(|data| {
                Rc::new(Segment::new(
                    data.start_vert,
                    data.end_vert,
                    data.angle,
                    Rc::clone(&linedefs[data.linedef]),
                    data.dir_like_linedef,
                    data.offset,
                ))
            })
            .collect();

        let subsectors: Vec<Rc<SubSector>> = data
            .subsectors
            .iter()
            .map(|data| {
                let mut segs = Vec::with_capacity(data.segment_count);
                for i in data.first_segment..data.first_segment + data.segment_count {
                    segs.push(Rc::clone(&segments[i]));
                }
                Rc::new(SubSector {
                    segment_count: data.segment_count,
                    segments: segs,
                })
            })
            .collect();

        let mut nodes: HashMap<i16, Rc<Node>> = HashMap::new();
        data.nodes.iter().enumerate().for_each(|(idx, data)| {
            let left = match data.left_child {
                ChildIdx::Subsector(s_idx) => {
                    Some(Child::SUBSECTOR(Rc::clone(&subsectors[s_idx as usize])))
                }
                ChildIdx::Node(n_idx) => Some(Child::NODE(Rc::clone(&nodes.get(&n_idx).unwrap()))),
            };
            let right = match data.right_child {
                ChildIdx::Subsector(s_idx) => {
                    Some(Child::SUBSECTOR(Rc::clone(&subsectors[s_idx as usize])))
                }
                ChildIdx::Node(n_idx) => Some(Child::NODE(Rc::clone(&nodes.get(&n_idx).unwrap()))),
            };
            let n = Rc::new(Node {
                partition_x: data.partition_x,
                partition_y: data.partition_y,
                delta_x: data.delta_x,
                delta_y: data.delta_y,
                right_bbox: data.right_bbox,
                left_bbox: data.left_bbox,
                right_child: left,
                left_child: right,
            });
            nodes.insert(idx as i16, n);
        });
        let root = Rc::clone(&nodes[&((nodes.len() - 1) as i16)]);
        Level {
            vertexes: data.vertexes.clone(),
            things: data.things.clone(),
            sectors,
            sidedefs,
            linedefs,
            segments,
            subsectors,
            nodes,
            root_node: root,
        }
    }
}
