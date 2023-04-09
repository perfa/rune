use regex::Regex;

#[derive(Clone, Debug)]
pub struct Header {
    pub id: String,
    pub num_lumps: usize,
    pub dir_offset: usize,
}

#[derive(Clone, Debug)]
pub struct FileLump {
    pub file_pos: usize,
    pub size: usize,
    pub name: String,
}

#[derive(Clone, Copy, Debug)]
pub struct Thing {
    pub x: i16,
    pub y: i16,
    pub angle_facing: f32,
    pub thing_type: i16,
    pub flags: i16,
}

#[derive(Clone, Copy, Debug)]
pub struct Linedef {
    pub start_vert: usize,
    pub end_vert: usize,
    pub flags: i16,
    pub special_type: i16,
    pub sector_tag: usize,
    pub right_sidedef: usize, // 65535 == None
    pub left_sidedef: usize,  // 65535 == None
}

#[derive(Clone, Debug)]
pub struct Sidedef {
    pub x_off: i16,
    pub y_off: i16,
    pub upper_tex: String,
    pub lower_tex: String,
    pub middle_tex: String,
    pub sector: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub start_vert: usize,
    pub end_vert: usize,
    pub angle: f32,
    pub linedef: usize,
    pub dir_like_linedef: bool,
    pub offset: i16, // May be u16 really
}

#[derive(Clone, Copy, Debug)]
pub struct SubSector {
    pub segment_count: usize,
    pub first_segment: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct BBox {
    pub top: i16,
    pub left: i16,
    pub width: i16,
    pub height: i16,
}

// Child pointer can identify node or leaf (SSeg)
#[derive(Clone, Copy, Debug)]
pub enum ChildIdx {
    Node(i16),
    Subsector(i16),
}

#[derive(Clone, Copy, Debug)]
pub struct MapNode {
    pub partition_x: i16,
    pub partition_y: i16,
    pub delta_x: i16,
    pub delta_y: i16,
    pub right_bbox: BBox,
    pub left_bbox: BBox,
    pub right_child: ChildIdx,
    pub left_child: ChildIdx,
    pub id: usize,
}

#[derive(Debug)]
pub enum Child {
    NODE(Box<Node>),
    SUBSECTOR(SubSector),
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
    pub fn new(n: &MapNode, map_nodes: &Vec<MapNode>, subsectors: Vec<SubSector>) -> Self {
        let mut new = Node {
            partition_x: n.partition_x,
            partition_y: n.partition_y,
            delta_x: n.delta_x,
            delta_y: n.delta_y,
            right_bbox: n.right_bbox,
            left_bbox: n.left_bbox,
            right_child: None,
            left_child: None,
        };
        match n.left_child {
            ChildIdx::Node(child_map) => {
                new.left_child = Some(Child::NODE(Box::new(Node::new(
                    map_nodes.get(child_map as usize).unwrap(),
                    map_nodes,
                    subsectors.iter().map(|ss| ss.clone()).collect(),
                ))))
            }
            ChildIdx::Subsector(ssec) => {
                new.left_child = Some(Child::SUBSECTOR(
                    subsectors.get(ssec as usize).unwrap().clone(),
                ))
            }
        }
        match n.right_child {
            ChildIdx::Node(child_map) => {
                new.right_child = Some(Child::NODE(Box::new(Node::new(
                    map_nodes.get(child_map as usize).unwrap(),
                    map_nodes,
                    subsectors.iter().map(|ss| ss.clone()).collect(),
                ))))
            }
            ChildIdx::Subsector(ssec) => {
                let x = subsectors[ssec as usize].clone();
                new.right_child = Some(Child::SUBSECTOR(x))
            }
        }

        new
    }

    pub fn find(&self, x: i16, y: i16) -> &SubSector {
        let child = if self.is_point_behind(x, y) {
            self.left_child.as_ref().unwrap()
        } else {
            self.right_child.as_ref().unwrap()
        };
        match child {
            Child::NODE(n) => n.find(x, y),
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

#[derive(Clone, Debug)]
pub struct Sector {
    pub floor_height: i16,
    pub ceiling_height: i16,
    pub floor_tex: String,
    pub ceiling_tex: String,
    pub light_level: i16,
    pub special_type: i16,
    pub tag: usize,
}

pub struct LevelData {
    pub things: Vec<Thing>,
    pub linedefs: Vec<Linedef>,
    pub sidedefs: Vec<Sidedef>,
    pub vertexes: Vec<Vertex>,
    pub segs: Vec<Segment>,
    pub subsectors: Vec<Box<SubSector>>,
    pub nodes: Node,
    pub sectors: Vec<Sector>,
}

pub struct WadFile {
    pub bytes: Vec<u8>,
    pub header: Header,
    pub directory: Vec<FileLump>,
    pub levels: Vec<LevelData>,
}

impl WadFile {
    fn get_i16(bytes: &[u8]) -> i16 {
        i16::from_le_bytes([bytes[0], bytes[1]])
    }

    fn get_child(bytes: &[u8]) -> ChildIdx {
        let val = u16::from_le_bytes([bytes[0], bytes[1]]);
        if val & 0x8000 == 0 {
            ChildIdx::Node(val as i16)
        } else {
            if val == 0xFFFF {
                ChildIdx::Subsector(0)
            } else {
                ChildIdx::Subsector((val ^ 0x8000) as i16)
            }
        }
    }

    fn get_i32(bytes: &[u8]) -> i32 {
        i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn get_angle(bytes: &[u8]) -> f32 {
        let binary_angle = u16::from_le_bytes([bytes[0], bytes[1]]);
        (360_f32 * f32::from(binary_angle) / f32::from(u16::MAX)).to_radians()
    }

    fn get_8char_string(bytes: &[u8]) -> String {
        let mut name_nul = 0;
        for idx in 0..8 {
            if bytes[idx] == 0 {
                name_nul = idx;
                break;
            }
            name_nul += 1;
        }
        std::str::from_utf8(&bytes[0..name_nul])
            .unwrap_or("????")
            .to_string()
    }

    fn get_bbox(bytes: &[u8]) -> BBox {
        let top = WadFile::get_i16(&bytes[0..2]);
        let bottom = WadFile::get_i16(&bytes[2..4]);
        let left = WadFile::get_i16(&bytes[4..6]);
        let right = WadFile::get_i16(&bytes[6..8]);
        BBox {
            left,
            top,
            width: right - left,
            height: top - bottom,
        }
    }

    pub fn load_from(path: &str) -> Self {
        let bytes = std::fs::read(path).unwrap();

        let header = Header {
            id: std::str::from_utf8(&bytes[0..4])
                .unwrap_or("IWAD")
                .to_string(),
            num_lumps: WadFile::get_i32(&bytes[4..8]) as usize,
            dir_offset: WadFile::get_i32(&bytes[8..12]) as usize,
        };

        let mut directory = Vec::with_capacity(header.num_lumps);
        for lump_idx in 0..header.num_lumps {
            let entry_idx = header.dir_offset + lump_idx * 16;
            let name_idx = entry_idx + 8;

            directory.push(FileLump {
                file_pos: WadFile::get_i32(&bytes[entry_idx..entry_idx + 4]) as usize,
                size: WadFile::get_i32(&bytes[entry_idx + 4..entry_idx + 8]) as usize,
                name: WadFile::get_8char_string(&bytes[name_idx..name_idx + 8]),
            })
        }

        // println!("Directory\n=========");
        // directory.iter().for_each(|lump| {
        //     println!(
        //         "* {} - Offset {} - Size {}",
        //         lump.name, lump.file_pos, lump.size
        //     );
        // });

        let mut levels = Vec::with_capacity(9);
        let re = Regex::new(r"^E[1234]M[0-9]").unwrap();
        let mut lump_idx: usize = 0;
        while lump_idx < directory.len() {
            let lump = &directory[lump_idx];
            if !re.is_match(lump.name.as_str()) {
                lump_idx += 1;
                continue;
            }
            //I'm A Level!
            lump_idx += 1;
            let things_lump = &directory[lump_idx];
            debug_assert!(things_lump.name == "THINGS");
            let mut things: Vec<Thing> = Vec::with_capacity(things_lump.size / 10); // 10 bytes/each
            for thing_idx in 0..things.capacity() {
                let thing_offset = things_lump.file_pos + thing_idx * 10;
                things.push(Thing {
                    x: WadFile::get_i16(&bytes[thing_offset..thing_offset + 2]),
                    y: WadFile::get_i16(&bytes[thing_offset + 2..thing_offset + 4]),
                    angle_facing: f32::from(WadFile::get_i16(
                        &bytes[thing_offset + 4..thing_offset + 6],
                    ))
                    .to_radians(),
                    thing_type: WadFile::get_i16(&bytes[thing_offset + 6..thing_offset + 8]),
                    flags: WadFile::get_i16(&bytes[thing_offset + 8..thing_offset + 10]),
                })
            }

            lump_idx += 1;
            let linedefs_lump = &directory[lump_idx];
            debug_assert!(linedefs_lump.name == "LINEDEFS");
            let mut linedefs: Vec<Linedef> = Vec::with_capacity(linedefs_lump.size / 14); // 14 bytes/each
            for linedef_idx in 0..linedefs.capacity() {
                let linedef_offset = linedefs_lump.file_pos + linedef_idx * 14;
                linedefs.push(Linedef {
                    start_vert: WadFile::get_i16(&bytes[linedef_offset..linedef_offset + 2])
                        as usize,
                    end_vert: WadFile::get_i16(&bytes[linedef_offset + 2..linedef_offset + 4])
                        as usize,
                    flags: WadFile::get_i16(&bytes[linedef_offset + 4..linedef_offset + 6]),
                    special_type: WadFile::get_i16(&bytes[linedef_offset + 6..linedef_offset + 8]),
                    sector_tag: WadFile::get_i16(&bytes[linedef_offset + 8..linedef_offset + 10])
                        as usize,
                    right_sidedef: WadFile::get_i16(
                        &bytes[linedef_offset + 10..linedef_offset + 12],
                    ) as usize,
                    left_sidedef: WadFile::get_i16(&bytes[linedef_offset + 12..linedef_offset + 14])
                        as usize,
                })
            }

            lump_idx += 1;
            let sidedefs_lump = &directory[lump_idx];
            debug_assert!(sidedefs_lump.name == "SIDEDEFS");
            let mut sidedefs: Vec<Sidedef> = Vec::with_capacity(sidedefs_lump.size / 30); // 30 bytes/each
            for sidedef_idx in 0..sidedefs.capacity() {
                let sidedef_offset = sidedefs_lump.file_pos + sidedef_idx * 30;
                sidedefs.push(Sidedef {
                    x_off: WadFile::get_i16(&bytes[sidedef_offset..sidedef_offset + 2]),
                    y_off: WadFile::get_i16(&bytes[sidedef_offset + 2..sidedef_offset + 4]),
                    upper_tex: WadFile::get_8char_string(
                        &bytes[sidedef_offset + 4..sidedef_offset + 12],
                    ),
                    lower_tex: WadFile::get_8char_string(
                        &bytes[sidedef_offset + 12..sidedef_offset + 20],
                    ),
                    middle_tex: WadFile::get_8char_string(
                        &bytes[sidedef_offset + 20..sidedef_offset + 28],
                    ),
                    sector: WadFile::get_i16(&bytes[sidedef_offset + 28..sidedef_offset + 30])
                        as usize,
                })
            }

            lump_idx += 1;
            let vertexes_lump = &directory[lump_idx];
            debug_assert!(vertexes_lump.name == "VERTEXES");
            let mut vertexes: Vec<Vertex> = Vec::with_capacity(vertexes_lump.size / 4); // 4 bytes/each
            for vertex_idx in 0..vertexes.capacity() {
                let vertex_offset = vertexes_lump.file_pos + vertex_idx * 4;
                vertexes.push(Vertex {
                    x: WadFile::get_i16(&bytes[vertex_offset..vertex_offset + 2]),
                    y: WadFile::get_i16(&bytes[vertex_offset + 2..vertex_offset + 4]),
                })
            }

            lump_idx += 1;
            let seg_lump = &directory[lump_idx];
            debug_assert!(seg_lump.name == "SEGS");
            let mut segs: Vec<Segment> = Vec::with_capacity(seg_lump.size / 12); // 12 bytes/each
            for seg_idx in 0..segs.capacity() {
                let seg_offset = seg_lump.file_pos + seg_idx * 12;
                segs.push(Segment {
                    start_vert: WadFile::get_i16(&bytes[seg_offset..seg_offset + 2]) as usize,
                    end_vert: WadFile::get_i16(&bytes[seg_offset + 2..seg_offset + 4]) as usize,
                    angle: WadFile::get_angle(&bytes[seg_offset + 4..seg_offset + 6]),
                    linedef: WadFile::get_i16(&bytes[seg_offset + 6..seg_offset + 8]) as usize,
                    dir_like_linedef: WadFile::get_i16(&bytes[seg_offset + 8..seg_offset + 10])
                        != 0,
                    offset: WadFile::get_i16(&bytes[seg_offset + 10..seg_offset + 12]),
                })
            }

            lump_idx += 1;
            let subsector_lump = &directory[lump_idx];
            debug_assert!(subsector_lump.name == "SSECTORS");
            let mut subsectors: Vec<Box<SubSector>> = Vec::with_capacity(subsector_lump.size / 4); // 4 bytes/each
            for subsector_idx in 0..subsectors.capacity() {
                let subsector_offset = subsector_lump.file_pos + subsector_idx * 4;
                subsectors.push(Box::new(SubSector {
                    segment_count: WadFile::get_i16(&bytes[subsector_offset..subsector_offset + 2])
                        as usize,
                    first_segment: WadFile::get_i16(
                        &bytes[subsector_offset + 2..subsector_offset + 4],
                    ) as usize,
                }))
            }

            lump_idx += 1;
            let node_lump = &directory[lump_idx];
            debug_assert!(node_lump.name == "NODES");
            let mut map_nodes: Vec<MapNode> = Vec::with_capacity(node_lump.size / 28); // 28 bytes/each
            for node_idx in 0..map_nodes.capacity() {
                let node_offset = node_lump.file_pos + node_idx * 28;
                map_nodes.push(MapNode {
                    id: node_idx,
                    partition_x: WadFile::get_i16(&bytes[node_offset..node_offset + 2]),
                    partition_y: WadFile::get_i16(&bytes[node_offset + 2..node_offset + 4]),
                    delta_x: WadFile::get_i16(&bytes[node_offset + 4..node_offset + 6]),
                    delta_y: WadFile::get_i16(&bytes[node_offset + 6..node_offset + 8]),
                    right_bbox: WadFile::get_bbox(&bytes[node_offset + 8..node_offset + 16]),
                    left_bbox: WadFile::get_bbox(&bytes[node_offset + 16..node_offset + 24]),
                    right_child: WadFile::get_child(&bytes[node_offset + 24..node_offset + 26]),
                    left_child: WadFile::get_child(&bytes[node_offset + 26..node_offset + 28]),
                })
            }

            let root_map_node = map_nodes.pop().unwrap();
            let nodes: Node = Node::new(
                &root_map_node,
                &map_nodes,
                subsectors.iter().map(|ss| *ss.clone()).collect(),
            );
            // println!("BSP: {:?}", nodes);

            lump_idx += 1;
            let sector_lump = &directory[lump_idx];
            debug_assert!(sector_lump.name == "SECTORS");
            let mut sectors: Vec<Sector> = Vec::with_capacity(sector_lump.size / 26); // 26 bytes/each
            for sector_idx in 0..sectors.capacity() {
                let sector_offset = sector_lump.file_pos + sector_idx * 26;
                sectors.push(Sector {
                    floor_height: WadFile::get_i16(&bytes[sector_offset..sector_offset + 2]),
                    ceiling_height: WadFile::get_i16(&bytes[sector_offset + 2..sector_offset + 4]),
                    floor_tex: WadFile::get_8char_string(
                        &bytes[sector_offset + 4..sector_offset + 12],
                    ),
                    ceiling_tex: WadFile::get_8char_string(
                        &bytes[sector_offset + 12..sector_offset + 20],
                    ),
                    light_level: WadFile::get_i16(&bytes[sector_offset + 20..sector_offset + 22]),
                    special_type: WadFile::get_i16(&bytes[sector_offset + 22..sector_offset + 24]),
                    tag: WadFile::get_i16(&bytes[sector_offset + 24..sector_offset + 26]) as usize,
                })
            }

            lump_idx += 1;
            let reject_lump = &directory[lump_idx];
            debug_assert!(reject_lump.name == "REJECT");
            // Ignored like a lord

            lump_idx += 1;
            let blockmap_lump = &directory[lump_idx];
            debug_assert!(blockmap_lump.name == "BLOCKMAP");
            // Ignored like a lord - but we'll need it at SOME point.

            levels.push(LevelData {
                things,
                linedefs,
                sidedefs,
                vertexes,
                segs,
                subsectors,
                nodes,
                sectors,
            });
        }

        WadFile {
            bytes,
            header,
            directory,
            levels,
        }
    }
}
