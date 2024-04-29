use colored::*;
use itertools::Itertools;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::{fmt, io};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bitset(pub u64);

pub type Board = Bitset;
pub type Placement = Bitset;

impl Bitset {
    pub fn empty() -> Bitset {
        Bitset(0)
    }

    pub fn from_orientation(orientation: &Orientation, dim: &Coord) -> Bitset {
        let mut mask = Bitset(0);
        for coord in &orientation.0 {
            mask.0 |= 1 << coord.z * dim.y * dim.x + coord.y * dim.x + coord.x
        }
        mask
    }

    pub fn get(&self, index: usize) -> bool {
        (self.0 >> index) & 1 == 1
    }

    pub fn set(&mut self, index: usize) {
        self.0 |= 1 << index;
    }

    pub fn intersects(&self, other: Bitset) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn xor(&self, other: Bitset) -> Bitset {
        Bitset(self.0 ^ other.0)
    }

    pub fn union(&self, other: Bitset) -> Bitset {
        Bitset(self.0 | other.0)
    }

    pub fn intersection(&self, other: Bitset) -> Bitset {
        Bitset(self.0 & other.0)
    }
}

#[derive(Clone)]
pub struct Piece {
    pub name: String,
    pub id: String,
    pub base: Orientation,
    pub placements: Vec<Placement>,
}

impl PartialEq for Piece {
    fn eq(&self, other: &Self) -> bool {
        // Equality based on the bitmask
        self.name == other.name
    }
}

impl Eq for Piece {}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Piece {{ {} }}", self.name)
    }
}
impl Piece {
    fn new(name: String, id: String, base: Orientation) -> Piece {
        Piece {
            name,
            id,
            base,
            placements: vec![],
        }
    }

    pub fn placements(&self) -> &Vec<Placement> {
        &self.placements
    }

    fn orientations(&self) -> Vec<Orientation> {
        // Has six faces
        // Each face can be in four rotations
        // Good resource: https://www.euclideanspace.com/maths/geometry/rotations/euler/examples/index.htm
        //      Matrix rep: https://www.euclideanspace.com/maths/algebra/matrix/transforms/examples/index.htm
        let mut current_orientation = self.base.clone();
        let mut orientations: Vec<Orientation> = vec![];
        for _ in 0..4 {
            orientations.push(current_orientation.clone());
            let mut o = current_orientation.clone();
            o.rotate(0, 1, 0);
            orientations.push(o);
            let mut o = current_orientation.clone();
            o.rotate(0, 3, 0);
            orientations.push(o);
            let mut o = current_orientation.clone();
            o.rotate(0, 0, 1);
            orientations.push(o);
            let mut o = current_orientation.clone();
            o.rotate(0, 0, 2);
            orientations.push(o);
            let mut o = current_orientation.clone();
            o.rotate(0, 0, 3);
            orientations.push(o);

            current_orientation.rotate(1, 0, 0);
        }
        let unique_orientations: Vec<Orientation> =
            orientations.iter().unique().map(|x| x.clone()).collect();
        unique_orientations
    }
}

#[derive(Clone, Eq, Debug)]
pub struct Orientation(Vec<Coord>);

impl Hash for Orientation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Get the bitmask and feed it into the hasher
        let dim = Coord::new(4, 4, 4);
        let placement = Placement::from_orientation(self, &dim);
        placement.0.hash(state);
    }
}

impl PartialEq for Orientation {
    fn eq(&self, other: &Self) -> bool {
        // Equality based on the bitmask
        let dim = Coord::new(4, 4, 4);
        let placement_a = Placement::from_orientation(self, &dim);
        let placement_b = Placement::from_orientation(other, &dim);
        placement_a.0 == placement_b.0
    }
}

impl Orientation {
    fn rotate(&mut self, x: usize, y: usize, z: usize) {
        // Rotate
        for _ in 0..x {
            self.0.iter_mut().for_each(|coord| coord.rotate_x());
        }
        for _ in 0..y {
            self.0.iter_mut().for_each(|coord| coord.rotate_y());
        }
        for _ in 0..z {
            self.0.iter_mut().for_each(|coord| coord.rotate_z());
        }

        // Normalise
        let min_x = self.0.iter().map(|coord| coord.x).min().unwrap();
        let min_y = self.0.iter().map(|coord| coord.y).min().unwrap();
        let min_z = self.0.iter().map(|coord| coord.z).min().unwrap();

        self.0
            .iter_mut()
            .for_each(|coord| coord.x = coord.x - min_x);
        self.0
            .iter_mut()
            .for_each(|coord| coord.y = coord.y - min_y);
        self.0
            .iter_mut()
            .for_each(|coord| coord.z = coord.z - min_z);
    }

    pub fn offset(&self) -> Coord {
        Coord {
            x: self.0.iter().map(|c| c.x).min().unwrap(),
            y: self.0.iter().map(|c| c.y).min().unwrap(),
            z: self.0.iter().map(|c| c.z).min().unwrap(),
        }
    }

    pub fn bounds(&self) -> Coord {
        Coord {
            x: self.0.iter().map(|c| c.x).max().unwrap(),
            y: self.0.iter().map(|c| c.y).max().unwrap(),
            z: self.0.iter().map(|c| c.z).max().unwrap(),
        }
    }

    pub fn normalise(&self) -> Orientation {
        let offset = self.offset();
        let blocks = self
            .0
            .iter()
            .map(|c| Coord {
                x: c.x - offset.x,
                y: c.y - offset.y,
                z: c.z - offset.z,
            })
            .collect();
        Orientation(blocks)
    }

    fn _rotate(&self, dir: Direction) -> Self {
        let mut ori = self.clone();
        for block in ori.0.iter_mut() {
            match dir {
                Direction::Next => {
                    let tmp = block.y;
                    block.y = block.z;
                    block.z = -tmp;
                }
                Direction::Clk => {
                    let tmp = block.x;
                    block.x = block.z;
                    block.z = -tmp;
                }
                Direction::CClk => {
                    let tmp = block.z;
                    block.z = block.x;
                    block.x = -tmp;
                }
            }
        }
        ori
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Coord {
    x: i64,
    y: i64,
    z: i64,
}

impl Coord {
    pub fn new(x: usize, y: usize, z: usize) -> Coord {
        Coord {
            x: x as i64,
            y: y as i64,
            z: z as i64,
        }
    }

    fn from_str(s: &str) -> Vec<Coord> {
        s.split("-")
            .map(|coord_s| {
                let mut coord_ids = coord_s.chars();
                Coord {
                    x: coord_ids.next().unwrap().to_digit(10).unwrap() as i64,
                    y: coord_ids.next().unwrap().to_digit(10).unwrap() as i64,
                    z: coord_ids.next().unwrap().to_digit(10).unwrap() as i64,
                }
            })
            .collect()
    }

    fn rotate_x(&mut self) {
        // [ 1  0  0
        //   0  0 -1
        //   0  1  0 ]
        let new_x = self.x;
        let new_y = -self.z;
        let new_z = self.y;
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
    }
    fn rotate_y(&mut self) {
        // [ 0  0  1
        //   0  1  0
        //  -1  0  0 ]
        let new_x = self.z;
        let new_y = self.y;
        let new_z = -self.x;
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
    }
    fn rotate_z(&mut self) {
        // [ 0 -1  0
        //   1  0  0
        //   0  0  1 ]
        let new_x = -self.y;
        let new_y = self.x;
        let new_z = self.z;
        self.x = new_x;
        self.y = new_y;
        self.z = new_z;
    }
}

enum Direction {
    Next,
    Clk,
    CClk,
}

pub struct Puzzle {
    pub name: String,
    pub pieces: Vec<Piece>,
    pub dim: Coord,
    pub full: Bitset,
}

impl Puzzle {
    pub fn from_csv(path: PathBuf) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut pieces = vec![];
        for (idx, result) in rdr.records().enumerate() {
            let record = result?;
            let color = record[1].parse().unwrap_or(Color::BrightRed);
            pieces.push(Piece::new(
                record[0].color(color).to_string(),
                format!("{:X}", idx).color(color).to_string(),
                Orientation(Coord::from_str(&record[2])),
            ));
        }

        let mut blocks = 0;
        for piece in &mut pieces {
            blocks += piece.base.0.len();
        }

        let d = (blocks as f64).cbrt().round() as usize;
        let dim = Coord::new(d, d, d);

        for piece in &mut pieces {
            piece.placements = Self::piece_placements(piece, &dim);
        }

        let full = Self::full(&dim);

        Ok(Puzzle {
            name: "Bedlam Cube".to_string(),
            pieces,
            dim,
            full,
        })
    }

    pub fn show(&self, arrangement: &Arrangement) {
        for y in (0..self.dim.y).rev() {
            for z in 0..self.dim.z {
                for x in 0..self.dim.x {
                    let index = z * self.dim.y * self.dim.x + y * self.dim.x + x;
                    if arrangement.occupied.get(index as usize) {
                        for (pid, placement) in arrangement.placements.iter() {
                            if placement.get(index as usize) {
                                print!("{} ", self.pieces[*pid].id);
                                break;
                            }
                        }
                    } else {
                        print!(". ");
                    }
                }
                print!("  ");
            }
            println!();
        }
    }

    pub fn show_bit(&self, bits: &Bitset) {
        for y in (0..self.dim.y).rev() {
            for z in 0..self.dim.z {
                for x in 0..self.dim.x {
                    let index = z * self.dim.y * self.dim.x + y * self.dim.x + x;
                    if bits.get(index as usize) {
                        print!("X ");
                    } else {
                        print!(". ");
                    }
                }
                print!("  ");
            }
            println!();
        }
    }

    pub fn rotate_within(&self, base: &Orientation) -> Vec<Orientation> {
        let mut orintations = Vec::new();
        // let mut ori = self.normalise();
        let mut ori = base.clone();
        orintations.push(ori.clone());
        let mut clk = true;
        for _dir in 0..6 {
            for _rot in 0..3 {
                ori = if clk {
                    ori._rotate(Direction::Clk)
                } else {
                    ori._rotate(Direction::CClk)
                };

                ori = Orientation(
                    ori.0
                        .iter()
                        .map(|c| Coord {
                            x: c.x.rem_euclid(self.dim.x),
                            y: c.y.rem_euclid(self.dim.y),
                            z: c.z.rem_euclid(self.dim.z),
                        })
                        .collect(),
                );

                println!("Rot: {:?}", ori);
                // if orintations.iter().all(|o| !o.similar(&ori)) {
                orintations.push(ori.clone());
                // }
            }
            ori = ori._rotate(Direction::Next).normalise();
            // if orintations.iter().all(|o| !o.similar(&ori)) {
            orintations.push(ori.clone());
            // }
            clk = !clk;
        }
        // orintations.iter().map(|o| o.normalise_first()).collect()
        orintations
    }

    pub fn piece_placements(piece: &Piece, dim: &Coord) -> Vec<Placement> {
        piece
            .orientations()
            .iter()
            .flat_map(|ori| Self::unique_placements(ori, dim))
            .collect()
    }

    pub fn unique_placements(ori: &Orientation, dim: &Coord) -> Vec<Placement> {
        let mut placements = vec![];
        let bounds = ori.bounds();
        for x_off in 0..(dim.x - bounds.x) {
            for y_off in 0..(dim.y - bounds.y) {
                for z_off in 0..(dim.z - bounds.z) {
                    let mut new_pos = ori.clone();
                    new_pos.0.iter_mut().for_each(|coord| {
                        coord.x += x_off;
                        coord.y += y_off;
                        coord.z += z_off;
                    });
                    placements.push(Placement::from_orientation(&new_pos, dim));
                }
            }
        }
        placements
    }

    pub fn full(dim: &Coord) -> Bitset {
        let mut full = Bitset::empty();
        for x in 0..dim.x {
            for y in 0..dim.y {
                for z in 0..dim.z {
                    let index = z * dim.y * dim.x + y * dim.x + x;
                    full.set(index as usize);
                }
            }
        }
        full
    }
}

#[derive(Clone)]
pub struct Arrangement {
    pub occupied: Bitset,
    pub placements: Vec<(usize, Bitset)>,
}

impl Arrangement {
    pub fn new() -> Arrangement {
        Arrangement {
            occupied: Bitset::empty(),
            placements: vec![],
        }
    }

    pub fn push(&mut self, piece: usize, placement: Bitset) {
        self.occupied = self.occupied.union(placement);
        self.placements.push((piece, placement));
    }

    pub fn pop(&mut self) -> Option<(usize, Bitset)> {
        match self.placements.pop() {
            Some((piece, placement)) => {
                self.occupied = self.occupied.xor(placement);
                Some((piece, placement))
            }
            None => None,
        }
    }
}
