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
            x: x,
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
        self.x.cmp(&other.x).then(self.y.cmp(&other.y))
    }
}
