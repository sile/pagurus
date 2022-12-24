use pagurus::random::StdRng;
use pagurus::spatial::{Contains, Position, Size};
use rand::Rng;
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HighScore(pub u8);

#[derive(Debug, Clone)]
pub struct GameState {
    pub snake: Snake,
    pub apple: Position,
}

impl Default for GameState {
    fn default() -> Self {
        let mut rng = StdRng::from_clock_seed(Duration::from_secs(0));
        Self::new(&mut rng)
    }
}

impl GameState {
    pub const BOARD_SIZE: Size = Size::square(10);
    pub const INITIAL_SNAKE_POSITION: Position = Position::from_xy(5, 5);

    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let mut this = Self {
            snake: Snake::new(Self::INITIAL_SNAKE_POSITION),
            apple: Position::ORIGIN,
        };
        this.spawn_apple(rng);
        this
    }

    pub fn score(&self) -> u8 {
        self.snake.tail.len() as u8
    }

    pub fn move_snake<R: Rng>(&mut self, rng: &mut R, direction: Direction) -> MoveResult {
        if !self.can_snake_move(direction) {
            return MoveResult::Crashed;
        }

        self.snake.tail.push_front(self.snake.head);
        self.snake.head = direction.move_position(self.snake.head);
        if self.snake.head == self.apple {
            self.spawn_apple(rng);
            MoveResult::Ate
        } else {
            self.snake.tail.pop_back();
            MoveResult::Moved
        }
    }

    fn can_snake_move(&self, direction: Direction) -> bool {
        let new_head = direction.move_position(self.snake.head);
        Self::BOARD_SIZE.contains(&new_head)
            && self
                .snake
                .tail
                .iter()
                .rev()
                .skip(1)
                .copied()
                .all(|p| p != new_head)
    }

    fn spawn_apple<R: Rng>(&mut self, rng: &mut R) {
        loop {
            self.apple = Position::from_xy(
                rng.gen_range(0..Self::BOARD_SIZE.width) as i32,
                rng.gen_range(0..Self::BOARD_SIZE.height) as i32,
            );
            if !self.snake.is_collision(self.apple) {
                break;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Snake {
    pub head: Position,
    pub tail: VecDeque<Position>,
}

impl Snake {
    fn new(head: Position) -> Self {
        Self {
            head,
            tail: VecDeque::new(),
        }
    }

    fn is_collision(&self, pos: Position) -> bool {
        self.head == pos || self.tail.iter().any(|p| *p == pos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn move_position(self, mut pos: Position) -> Position {
        match self {
            Self::Up => pos.y -= 1,
            Self::Down => pos.y += 1,
            Self::Left => pos.x -= 1,
            Self::Right => pos.x += 1,
        }
        pos
    }

    pub fn reverse(self) -> Self {
        match self {
            Direction::Up => Self::Down,
            Direction::Down => Self::Up,
            Direction::Left => Self::Right,
            Direction::Right => Self::Left,
        }
    }
}

#[derive(Debug)]
pub enum MoveResult {
    Moved,
    Crashed,
    Ate,
}
