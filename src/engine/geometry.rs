use engine::constants::*;

//TODO: Change Point to be a single number, or stored as a bitfield
// (bitfield to x and y for writing move might be hard?

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub index: u8
}

impl Point {
    pub fn new(x: u8, y: u8) -> Point {
        let flipped_x = if x >= SINGLE_MAP_WIDTH {
            FULL_MAP_WIDTH - x - 1
        } else {
            x
        };
        Point {
            index: y * SINGLE_MAP_WIDTH + flipped_x
        }
    }

    pub fn new_double_bitfield(x: u8, y: u8, is_left_player: bool) -> (u64, u64) {
        let bitfield = Point::new(x, y).to_either_bitfield();
        if (x >= SINGLE_MAP_WIDTH) == is_left_player {
            (0, bitfield)
        } else {
            (bitfield, 0)
        }
    }

    pub fn x(&self) -> u8 {
        self.index % SINGLE_MAP_WIDTH
    }

    pub fn y(&self) -> u8 {
        self.index / SINGLE_MAP_WIDTH
    }
}

impl Point {
    /**
     * # Bitfields
     * 
     * 0,0 is the top left point.
     * >> (towards 0) moves bits towards the player that owns that side
     * << (towards max) moves bits towards the opponent
     * This involves mirroring the x dimension for the opponent's side
     */

    pub fn to_either_bitfield(&self) -> u64 {
        1u64 << self.index
    }
}

use std::cmp::Ord;
use std::cmp::Ordering;

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Point {
    fn cmp(&self, other: &Point) -> Ordering {
        self.index.cmp(&other.index)
    }
}
