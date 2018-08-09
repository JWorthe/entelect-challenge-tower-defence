use engine::constants::*;

//TODO: Change Point to be a single number, or stored as a bitfield
// (bitfield to x and y for writing move might be hard?

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: u8,
    pub y: u8
}

impl Point {
    pub fn new(x: u8, y: u8) -> Point {
        Point { x, y }
    }
    pub fn move_left(&self) -> Option<Point> {
        self.x.checked_sub(1).map(|x| Point {
            x,
            ..*self
        })
    }
    pub fn move_right(&self, size: &Point) -> Option<Point> {
        if self.x + 1 >= size.x {
            None
        } else {
            Some(Point {
                x: self.x + 1,
                ..*self
            })
        }
    }

    pub fn wrapping_move_left(&mut self) {
        self.x = self.x.wrapping_sub(1);
    }
    pub fn wrapping_move_right(&mut self) {
        self.x = self.x.wrapping_add(1);
    }

    pub fn flip_x(&self) -> Point {
        let flipped_x = if self.x >= SINGLE_MAP_WIDTH {
            FULL_MAP_WIDTH - self.x - 1
        } else {
            self.x
        };
        Point::new(flipped_x, self.y)
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


    pub fn to_bitfield(&self) -> (u64, u64) {
        (self.to_left_bitfield(), self.to_right_bitfield())
    }
    
    pub fn to_left_bitfield(&self) -> u64 {
        if self.x >= SINGLE_MAP_WIDTH {
            0
        } else {
            let index = self.y * SINGLE_MAP_WIDTH + self.x;
            1 << index
        }
    }

    pub fn to_right_bitfield(&self) -> u64 {
        if self.x < SINGLE_MAP_WIDTH {
            0
        } else {
            let index = self.y * SINGLE_MAP_WIDTH + FULL_MAP_WIDTH - self.x - 1;
            1 << index
        }
    }

    pub fn to_either_bitfield(&self) -> u64 {
        self.to_left_bitfield() | self.to_right_bitfield()
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
        let a = self.flip_x();
        let b = other.flip_x();
        a.y.cmp(&b.y).then(a.x.cmp(&b.x))
    }
}
