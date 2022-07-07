use std::{cmp::Ordering, ops::Add};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const fn from_wh(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub const fn square(size: u32) -> Self {
        Self::from_wh(size, size)
    }

    pub fn len(self) -> usize {
        (self.width * self.height) as usize
    }

    pub fn iter(self) -> impl Iterator<Item = Position> {
        (0..self.height as i32)
            .flat_map(move |y| (0..self.width as i32).map(move |x| Position { x, y }))
    }

    pub fn contains(self, pos: Position) -> bool {
        (0..self.width as i32).contains(&pos.x) && (0..self.height as i32).contains(&pos.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub const ORIGIN: Self = Self { x: 0, y: 0 };

    pub const fn from_xy(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    // TODO: rename (move?)
    pub fn shift_x(mut self, delta: i32) -> Self {
        self.x += delta;
        self
    }

    pub fn shift_y(mut self, delta: i32) -> Self {
        self.y += delta;
        self
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_xy(self.x + rhs.x, self.y + rhs.y)
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.x == other.x && self.y == other.y {
            Some(Ordering::Equal)
        } else if self.x <= other.x && self.y <= other.y {
            Some(Ordering::Less)
        } else if self.x >= other.x && self.y >= other.y {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Region {
    pub position: Position,
    pub size: Size,
}

impl Region {
    pub const fn new(position: Position, size: Size) -> Self {
        Self { position, size }
    }

    pub fn contains(self, other: Self) -> bool {
        self.start() <= other.start() && other.end() <= self.end()
    }

    pub const fn start(self) -> Position {
        self.position
    }

    pub fn end(self) -> Position {
        Position::from_xy(
            self.position.x + self.size.width as i32,
            self.position.y + self.size.height as i32,
        )
    }

    pub fn shift_x(mut self, n: i32) -> Self {
        self.position.x += self.size.width as i32 * n;
        self
    }

    pub fn shift_y(mut self, n: i32) -> Self {
        self.position.y += self.size.height as i32 * n;
        self
    }
}

impl From<Size> for Region {
    fn from(size: Size) -> Self {
        Self::new(Position::ORIGIN, size)
    }
}
