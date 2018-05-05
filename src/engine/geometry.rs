#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: u8,
    pub y: u8
}

impl Point {
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
