//! The garage doors' and portcullises' rules (`GarageDoor`), over plain values so `cargo test`
//! checks them without a game: opening and closing a column of segments.

/// The four horizontal directions, in `Direction.Plane.HORIZONTAL` order.
pub const HORIZONTAL: [(&str, i32, i32); 4] = [
    ("north", 0, -1),
    ("east", 1, 0),
    ("south", 0, 1),
    ("west", -1, 0),
];

/// `GarageDoor.GaragePart`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Top,
    Middle,
}

impl Part {
    pub fn name(self) -> &'static str {
        match self {
            Part::Top => "top",
            Part::Middle => "middle",
        }
    }
}

/// `GarageDoor.getStateForPlacement`: `getHorizontalDirection().getClockWise()`, the horizontal
/// direction being `Direction.fromYRot(yaw)`.
pub fn placed_facing(yaw: f32) -> &'static str {
    match ((f64::from(yaw) / 90.0 + 0.5).floor() as i64).rem_euclid(4) {
        0 => "west",
        1 => "north",
        2 => "east",
        _ => "south",
    }
}

/// One garage or portcullis block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seg {
    /// The block id, `mcwdoors:garage_white_door`.
    pub block: String,
    pub facing: String,
    pub open: bool,
    pub powered: bool,
    pub part: Part,
}

impl Seg {
    /// The state string the game parses.
    pub fn text(&self) -> String {
        format!(
            "{}[facing={},open={},part={},powered={}]",
            self.block,
            self.facing,
            self.open,
            self.part.name(),
            self.powered
        )
    }

    /// Read a state string of a garage block; `None` for any other block.
    pub fn parse(text: &str) -> Option<Seg> {
        let (block, props) = text.strip_suffix(']')?.split_once('[')?;
        let (mut facing, mut open, mut powered, mut part) = (None, None, None, None);
        for pair in props.split(',') {
            match pair.split_once('=')? {
                ("facing", v) => facing = Some(v.to_owned()),
                ("open", v) => open = Some(v == "true"),
                ("powered", v) => powered = Some(v == "true"),
                ("part", "top") => part = Some(Part::Top),
                ("part", "middle") => part = Some(Part::Middle),
                _ => {}
            }
        }
        Some(Seg {
            block: block.to_owned(),
            facing: facing?,
            open: open?,
            powered: powered?,
            part: part?,
        })
    }
}

/// What stands at a position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Air,
    Garage(Seg),
    Other,
}

/// The level the rules read and write.
pub trait World {
    fn at(&self, pos: [i32; 3]) -> Cell;
    fn put(&mut self, pos: [i32; 3], cell: Cell);
}

fn up([x, y, z]: [i32; 3]) -> [i32; 3] {
    [x, y + 1, z]
}

fn down([x, y, z]: [i32; 3]) -> [i32; 3] {
    [x, y - 1, z]
}

/// `GarageDoor.toggleDoor`: set the topmost segment above `pos` to `open`; closing it rolls
/// middle segments down through the air below, opening it removes them; then the same for each
/// horizontal neighbour that is the same block and in the other state.
pub fn toggle(world: &mut dyn World, pos: [i32; 3], open: bool, facing: &str) {
    let Cell::Garage(start) = world.at(pos) else {
        return;
    };
    let block = start.block;
    let same = |cell: &Cell| matches!(cell, Cell::Garage(s) if s.block == block);
    let mut top = pos;
    while same(&world.at(up(top))) {
        top = up(top);
    }
    let Cell::Garage(mut seg) = world.at(top) else {
        return;
    };
    seg.open = open;
    world.put(top, Cell::Garage(seg));
    let mut below = down(top);
    if !open {
        while world.at(below) == Cell::Air {
            world.put(
                below,
                Cell::Garage(Seg {
                    block: block.clone(),
                    facing: facing.to_owned(),
                    open: false,
                    powered: false,
                    part: Part::Middle,
                }),
            );
            below = down(below);
        }
    } else {
        while same(&world.at(below)) {
            world.put(below, Cell::Air);
            below = down(below);
        }
    }
    for (_, dx, dz) in HORIZONTAL {
        let next = [top[0] + dx, top[1], top[2] + dz];
        if let Cell::Garage(n) = world.at(next)
            && n.block == block
            && n.open != open
        {
            let facing = n.facing.clone();
            toggle(world, next, open, &facing);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct Grid(HashMap<[i32; 3], Cell>, i32);

    impl World for Grid {
        fn at(&self, pos: [i32; 3]) -> Cell {
            match self.0.get(&pos) {
                Some(c) => c.clone(),
                None if pos[1] > self.1 => Cell::Air,
                None => Cell::Other,
            }
        }
        fn put(&mut self, pos: [i32; 3], cell: Cell) {
            self.0.insert(pos, cell);
        }
    }

    fn seg(open: bool, part: Part) -> Cell {
        Cell::Garage(Seg {
            block: "mcwdoors:garage_white_door".into(),
            facing: "east".into(),
            open,
            powered: false,
            part,
        })
    }

    #[test]
    fn closing_rolls_the_door_down_to_the_floor() {
        // A top segment at y=5, ground below y=1: air at y 2..=4.
        let mut g = Grid(HashMap::new(), 1);
        g.put([0, 5, 0], seg(true, Part::Top));
        toggle(&mut g, [0, 5, 0], false, "east");
        assert_eq!(g.at([0, 5, 0]), seg(false, Part::Top));
        for y in 2..=4 {
            assert_eq!(g.at([0, y, 0]), seg(false, Part::Middle), "y={y}");
        }
        assert_eq!(g.at([0, 1, 0]), Cell::Other);
    }

    #[test]
    fn opening_removes_the_rolled_segments_whichever_one_was_clicked() {
        let mut g = Grid(HashMap::new(), 1);
        g.put([0, 5, 0], seg(true, Part::Top));
        toggle(&mut g, [0, 5, 0], false, "east");
        toggle(&mut g, [0, 3, 0], true, "east");
        assert_eq!(g.at([0, 5, 0]), seg(true, Part::Top));
        for y in 2..=4 {
            assert_eq!(g.at([0, y, 0]), Cell::Air, "y={y}");
        }
    }

    #[test]
    fn neighbours_of_the_same_block_follow() {
        let mut g = Grid(HashMap::new(), 1);
        g.put([0, 5, 0], seg(true, Part::Top));
        g.put([0, 5, 1], seg(true, Part::Top));
        g.put([0, 5, 2], seg(true, Part::Top));
        toggle(&mut g, [0, 5, 0], false, "east");
        for z in 0..3 {
            assert_eq!(g.at([0, 5, z]), seg(false, Part::Top), "z={z}");
            assert_eq!(g.at([0, 2, z]), seg(false, Part::Middle), "z={z}");
        }
    }

    #[test]
    fn another_colour_is_left_alone() {
        let mut g = Grid(HashMap::new(), 1);
        g.put([0, 5, 0], seg(true, Part::Top));
        let mut black = seg(true, Part::Top);
        if let Cell::Garage(s) = &mut black {
            s.block = "mcwdoors:garage_black_door".into();
        }
        g.put([0, 5, 1], black.clone());
        toggle(&mut g, [0, 5, 0], false, "east");
        assert_eq!(g.at([0, 5, 1]), black);
    }

    #[test]
    fn state_strings_round_trip() {
        let s = Seg::parse(
            "mcwdoors:iron_portcullis[facing=north,open=false,part=middle,powered=true]",
        )
        .unwrap();
        assert_eq!(s.part, Part::Middle);
        assert!(s.powered && !s.open);
        assert_eq!(Seg::parse(&s.text()), Some(s));
        assert_eq!(Seg::parse("minecraft:stone"), None);
    }
}
