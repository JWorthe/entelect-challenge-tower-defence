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

    pub fn to_bitfield(&self, width: u8) -> (u64, u64) {
        if self.x >= width {
            let index = self.y * width + self.x - width;
            (0, 1 << index)
        } else {
            let index = self.y * width + self.x;
            (1 << index, 0)
        }
    }
    
    pub fn to_left_bitfield(&self, width: u8) -> u64 {
        if self.x >= width {
            0
        } else {
            let index = self.y * width + self.x;
            1 << index
        }
    }

    pub fn to_right_bitfield(&self, width: u8) -> u64 {
        if self.x < width {
            0
        } else {
            let index = self.y * width + self.x - width;
            1 << index
        }
    }

    pub fn to_either_bitfield(&self, width: u8) -> u64 {
        if self.x >= width {
            let index = self.y * width + self.x - width;
            1 << index
        } else {
            let index = self.y * width + self.x;
            1 << index
        }
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
        self.y.cmp(&other.y).then(self.x.cmp(&other.x))
    }
}
