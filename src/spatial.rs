use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Sub},
};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const EMPTY: Self = Self::square(0);

    pub const fn from_wh(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub const fn square(size: u32) -> Self {
        Self::from_wh(size, size)
    }

    pub fn len(self) -> usize {
        (self.width * self.height) as usize
    }

    pub fn is_empty(self) -> bool {
        self.width == 0 && self.height == 0
    }

    pub fn iter(self) -> impl Iterator<Item = Position> {
        (0..self.height as i32)
            .flat_map(move |y| (0..self.width as i32).map(move |x| Position { x, y }))
    }

    pub fn aspect_ratio(self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub const fn to_region(self) -> Region {
        Region::new(Position::ORIGIN, self)
    }
}

impl Contains<Position> for Size {
    fn contains(&self, Position { x, y }: &Position) -> bool {
        (0..self.width as i32).contains(x) && (0..self.height as i32).contains(y)
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub const ORIGIN: Self = Self { x: 0, y: 0 };

    pub const fn from_xy(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_xy(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<i32> for Position {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        Self::from_xy(self.x + rhs, self.y + rhs)
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_xy(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<i32> for Position {
    type Output = Self;

    fn sub(self, rhs: i32) -> Self::Output {
        Self::from_xy(self.x - rhs, self.y - rhs)
    }
}

impl Mul<u32> for Position {
    type Output = Self;

    fn mul(self, scale: u32) -> Self::Output {
        Self::from_xy(self.x * scale as i32, self.y * scale as i32)
    }
}

impl Div<u32> for Position {
    type Output = Self;

    fn div(self, scale: u32) -> Self::Output {
        Self::from_xy(self.x / scale as i32, self.y / scale as i32)
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

    pub const fn start(self) -> Position {
        self.position
    }

    pub fn end(self) -> Position {
        Position::from_xy(
            self.position.x + self.size.width as i32,
            self.position.y + self.size.height as i32,
        )
    }

    pub fn iter(self) -> impl Iterator<Item = Position> {
        let start = self.start();
        let end = self.end();
        (start.y..end.y).flat_map(move |y| (start.x..end.x).map(move |x| Position::from_xy(x, y)))
    }

    pub fn shift_x(mut self, n: i32) -> Self {
        self.position.x += self.size.width as i32 * n;
        self
    }

    pub fn shift_y(mut self, n: i32) -> Self {
        self.position.y += self.size.height as i32 * n;
        self
    }

    pub fn intersection(mut self, other: Self) -> Self {
        self.position.x = std::cmp::max(self.position.x, other.position.x);
        self.position.y = std::cmp::max(self.position.y, other.position.y);
        self.size.width = std::cmp::min(self.size.width, other.size.width);
        self.size.height = std::cmp::min(self.size.height, other.size.height);
        self
    }
}

impl From<Size> for Region {
    fn from(size: Size) -> Self {
        Self::new(Position::ORIGIN, size)
    }
}

impl Contains<Region> for Region {
    fn contains(&self, target: &Self) -> bool {
        self.start() <= target.start() && target.end() <= self.end()
    }
}

impl Contains<Position> for Region {
    fn contains(&self, Position { x, y }: &Position) -> bool {
        let start = self.start();
        let end = self.end();
        (start.x..end.x).contains(x) && (start.y..end.y).contains(y)
    }
}

pub trait Contains<T> {
    fn contains(&self, target: &T) -> bool;
}
