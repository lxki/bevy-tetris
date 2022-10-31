use std::cmp::max;
use std::vec;
use std::{collections::HashMap, num::NonZeroU32};

mod blocks;
use blocks::*;
pub use blocks::{get_block_color, BlockType};

mod input;
pub use input::Input;
use input::SmartInput;

mod utils;
use utils::{IdGenerator, Timer};

mod rotate;
use rotate::rotate_block;

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 24;
pub const HIDDEN_BOARD_TOP: usize = 4;
pub const VISIBLE_BOARD_HEIGHT: usize = BOARD_HEIGHT - HIDDEN_BOARD_TOP;

pub(self) const WAIT_DURATION: u32 = 30;
pub(self) const REPEAT_DURATION: u32 = 5;

pub type Id = NonZeroU32;
pub type Position = (usize, usize);

pub fn add_positions(a: Position, b: Position) -> Position {
    (a.0 + b.0, a.1 + b.1)
}

pub enum TickChange {
    /// Active block is locked to the board.
    BlockLocked,
    /// New active block has arrived.
    NewBlock,
    /// Block points was removed.
    PointRemoved(Id),
}

#[derive(Clone, Copy)]
pub struct Point {
    pub id: Id,
    pub origin_block_type: BlockType,
}

pub struct Block {
    pub id: Id,
    pub block_type: BlockType,
    points: Vec<Point>,
    points_pos: HashMap<Id, Position>,
}

impl Block {
    fn new(id: Id, block_type: BlockType, gen_id: &mut IdGenerator) -> Self {
        let block_points = get_block_points(block_type);

        let mut points = Vec::with_capacity(block_points.len());
        let mut points_pos = HashMap::with_capacity(block_points.len());
        for &pos in block_points {
            let point = Point {
                id: gen_id(),
                origin_block_type: block_type,
            };

            points.push(point);
            points_pos.insert(point.id, pos);
        }

        Self {
            id,
            block_type,
            points,
            points_pos,
        }
    }

    pub(self) fn width(&self) -> usize {
        *self.points_pos.values().map(|(x, _)| x).max().unwrap()
    }

    pub(self) fn height(&self) -> usize {
        *self.points_pos.values().map(|(_, y)| y).max().unwrap()
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }

    pub fn get_point_position(&self, point_id: Id) -> Option<Position> {
        self.points_pos.get(&point_id).copied()
    }
}

struct GameRules {}

impl GameRules {
    fn new() -> Self {
        GameRules {}
    }

    fn drop_speed(&self) -> u32 {
        10
    }

    fn fast_drop_speed(&self) -> u32 {
        max(self.drop_speed() / 2, 1)
    }
}

pub struct Game {
    rules: GameRules,
    gen_id: IdGenerator,
    input: SmartInput,
    board: [[Option<Point>; BOARD_WIDTH]; BOARD_HEIGHT],
    points_pos: HashMap<Id, Position>,
    active_block: Block,
    active_block_pos: Position,
    drop_timer: Timer,
}

impl Game {
    pub fn new() -> Self {
        let mut gen_id = IdGenerator::new();
        let active_block = Block::new(gen_id(), get_random_block(), &mut gen_id);
        let active_block_pos = (4, 0);

        Self {
            rules: GameRules::new(),
            gen_id: gen_id,
            input: SmartInput::new(),
            points_pos: HashMap::new(),
            board: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            active_block: active_block,
            active_block_pos: active_block_pos,
            drop_timer: Timer::new(),
        }
    }

    pub fn active_block(&self) -> &Block {
        &self.active_block
    }

    pub fn active_block_position(&self) -> Position {
        self.active_block_pos
    }

    pub fn tick(&mut self, input: &dyn Input) -> Vec<TickChange> {
        let mut changes = vec![];
        let mut block_pos = self.active_block_pos;
        self.input.tick(input);

        if self.input.move_left() {
            if block_pos.0 > 0
                && !self.is_block_collides(
                    self.active_block.points_pos.values(),
                    (block_pos.0 - 1, block_pos.1),
                )
            {
                block_pos.0 -= 1;
            }
        }
        if self.input.move_right() {
            if block_pos.0 + self.active_block.width() < BOARD_WIDTH - 1
                && !self.is_block_collides(
                    self.active_block.points_pos.values(),
                    (block_pos.0 + 1, block_pos.1),
                )
            {
                block_pos.0 += 1;
            }
        }
        if self.input.rotate() {
            if let Some((new_points_pos, new_block_pos)) =
                rotate_block(&self.active_block, block_pos, |block_points, block_pos| {
                    !self.is_block_collides(block_points.iter(), block_pos)
                })
            {
                self.active_block.points_pos = new_points_pos;
                block_pos = new_block_pos;
            }
        }

        let drop_speed = if self.input.fast_drop() {
            self.rules.fast_drop_speed()
        } else {
            self.rules.drop_speed()
        };

        if self.drop_timer.tick_and_restart_if_elapsed(drop_speed) {
            if block_pos.1 + self.active_block.height() == BOARD_HEIGHT - 1
                || self.is_block_collides(
                    self.active_block.points_pos.values(),
                    (block_pos.0, block_pos.1 + 1),
                )
            {
                self.lock_active_block_to_board(block_pos);
                changes.push(TickChange::BlockLocked);

                let filled_rows = self.find_filled_rows();
                let removed_points = self.remove_rows(&filled_rows);
                for p in removed_points {
                    changes.push(TickChange::PointRemoved(p.id));
                }

                self.spawn_block();
                changes.push(TickChange::NewBlock);
            } else {
                block_pos.1 += 1;
                self.active_block_pos = block_pos;
            }
        } else {
            self.active_block_pos = block_pos;
        }

        changes
    }

    pub fn get_point_position(&self, point_id: Id) -> Option<Position> {
        self.points_pos.get(&point_id).copied()
    }

    /// Returns `true` if block will collide with any of board points.
    fn is_block_collides<'a>(
        &self,
        block_points: impl Iterator<Item = &'a Position>,
        block_pos: Position,
    ) -> bool {
        for &point_pos in block_points {
            let (x, y) = add_positions(block_pos, point_pos);
            if self.board[y][x].is_some() {
                return true;
            }
        }
        false
    }

    fn spawn_block(&mut self) {
        self.active_block = Block::new((self.gen_id)(), get_random_block(), &mut self.gen_id);
        self.active_block_pos = (4, 0);
    }

    fn lock_active_block_to_board(&mut self, block_pos: Position) {
        for point in self.active_block.points() {
            let point = point.clone();
            let (x, y) = add_positions(
                block_pos,
                self.active_block.get_point_position(point.id).unwrap(),
            );

            assert!(self.board[y][x].is_none());
            self.board[y][x] = Some(point);
            self.points_pos.insert(point.id, (x, y));
        }
    }

    fn find_filled_rows(&self) -> Vec<usize> {
        let mut rows = vec![];
        for y in 0..BOARD_HEIGHT {
            if self.board[y].iter().all(|p| p.is_some()) {
                rows.push(y);
            }
        }
        rows
    }

    fn remove_rows(&mut self, rows: &[usize]) -> Vec<Point> {
        let mut removed_points = vec![];

        let mut drop = 0;
        let mut i = rows.len();
        for y in (0..BOARD_HEIGHT).rev() {
            if i > 0 && rows[i - 1] == y {
                for x in 0..BOARD_WIDTH {
                    if let Some(p) = self.board[y][x].take() {
                        self.points_pos.remove(&p.id);
                        removed_points.push(p);
                    }
                }
                i -= 1;
                drop += 1;
            } else if drop > 0 {
                for x in 0..BOARD_WIDTH {
                    if let Some(p) = self.board[y][x].take() {
                        self.board[y + drop][x] = Some(p);
                        self.points_pos.insert(p.id, (x, y + drop));
                    }
                }
            }
        }

        removed_points
    }
}
