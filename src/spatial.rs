#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn len(self) -> usize {
        (self.width * self.height) as usize
    }

    pub fn iter(self) -> impl Iterator<Item = Position> {
        (0..self.height as i32)
            .flat_map(move |y| (0..self.width as i32).map(move |x| Position { x, y }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub const fn origin() -> Self {
        Self { x: 0, y: 0 }
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
}

impl From<Size> for Region {
    fn from(size: Size) -> Self {
        Self::new(Position::origin(), size)
    }
}
