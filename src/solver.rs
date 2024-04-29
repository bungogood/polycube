use num_format::{Locale, ToFormattedString};

use crate::puzzle::{Arrangement, Bitset, Board, Coord, Orientation, Placement, Puzzle};

use std::time::Instant;

pub struct Solver {
    pub explored: usize,
    pub solutions: Vec<Vec<(usize, Placement)>>,
    pub start_time: Option<Instant>,
    pub verbose: bool,
}

impl Solver {
    pub fn build(verbose: bool) -> Solver {
        Solver {
            explored: 0,
            solutions: Vec::new(),
            start_time: None,
            verbose,
        }
    }

    fn add_solution(&mut self, puzzle: &Puzzle, arrangement: Arrangement) {
        self.solutions.push(arrangement.placements.clone());

        if self.verbose {
            puzzle.show(&arrangement);

            let duration = self.start_time.unwrap().elapsed();
            let rate = duration / self.solutions.len() as u32;
            println!(
                "Solutions: {} Explored: {} [rate {:.2?} per solution]",
                self.solutions.len().to_formatted_string(&Locale::en),
                self.explored.to_formatted_string(&Locale::en),
                rate
            )
        }
    }

    pub fn has_full_coverage(&self, puzzle: &Puzzle, tmp: Bitset, pieces: &Vec<usize>) -> bool {
        let mut coverage = tmp.clone();
        for pid in pieces {
            let piece = &puzzle.pieces[*pid];
            for placement in piece.placements() {
                if !tmp.intersects(*placement) {
                    coverage = coverage.union(*placement);
                    if coverage == puzzle.full {
                        return true;
                    }
                }
            }
        }
        coverage == puzzle.full
    }

    pub fn can_pieces_fit(&self, puzzle: &Puzzle, tmp: Bitset, pieces: &Vec<usize>) -> bool {
        for pid in pieces {
            if puzzle.pieces[*pid]
                .placements
                .iter()
                .all(|placement: &Placement| !tmp.intersects(*placement))
            {
                return false;
            }
        }
        return true;
    }

    fn new_cube(
        &self,
        puzzle: &Puzzle,
        arrangement: &Arrangement,
        prev: usize,
    ) -> Option<(usize, Bitset)> {
        let mut cube = prev;
        let mut mask = 1 << cube;

        while mask & arrangement.occupied.0 != 0 {
            cube += 1;
            mask <<= 1;
        }

        // do a check to ensure not isolated cube

        Some((cube, Bitset(mask)))
    }

    fn solve_board(
        &mut self,
        puzzle: &Puzzle,
        arrangement: &mut Arrangement,
        prev: usize,
        remaining: &Vec<usize>,
    ) {
        self.explored += 1;

        if remaining.is_empty() {
            self.add_solution(puzzle, arrangement.clone());
            return;
        }

        let (cube, mask) = match self.new_cube(puzzle, arrangement, prev) {
            Some((c, m)) => (c, m),
            None => return,
        };

        for (idx, pid) in remaining.iter().enumerate() {
            let mut other_pieces = remaining.clone();
            other_pieces.remove(idx);
            let piece = &puzzle.pieces[*pid];

            for &placement in piece.placements() {
                let new_board = arrangement.occupied.union(placement);
                if !arrangement.occupied.intersects(placement)
                    && placement.intersects(mask)
                    && self.has_full_coverage(puzzle, new_board, &other_pieces)
                    && self.can_pieces_fit(puzzle, new_board, &other_pieces)
                {
                    arrangement.push(*pid, placement);
                    self.solve_board(puzzle, arrangement, cube, &other_pieces);
                    arrangement.pop();
                }
            }
        }
    }

    pub fn begin(&mut self, puzzle: &Puzzle) {
        self.start_time = Some(Instant::now());
        let mut arrangement = Arrangement::new();

        let (cid, contrained) = puzzle
            .pieces
            .iter()
            .enumerate()
            .min_by_key(|(_, p)| p.placements.len())
            .unwrap();

        // println!("Constrained piece: {:?}", contrained);

        // for (idx, rot) in puzzle.rotate_within(&contrained.base).iter().enumerate() {
        //     println!("Rot: {}", idx);
        //     let bits = Bitset::from_orientation(rot);
        //     puzzle.show_bit(&bits);
        // }

        let placements = vec![Bitset(0x0000000000000272), Bitset(0x0000000002720000)];

        let remaining = (0..puzzle.pieces.len()).filter(|&x| x != cid).collect();

        for placement in placements {
            arrangement.push(cid, placement);
            self.solve_board(puzzle, &mut arrangement, 0, &remaining);
            arrangement.pop();
        }

        // let remaining = (0..puzzle.pieces.len()).collect();
        // self.solve_board(puzzle, &mut arrangement, 0, &remaining);
    }
}
