use std::cmp::max;
use std::collections::HashMap;
use std::num::NonZeroU32;

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;

const WAIT_DURATION: u32 = 30;
const REPEAT_DURATION: u32 = 5;

/*
 {
        vec![
            (vec![(0, 0), (0, 1), (1, 1), (1, 2)], Color::RED),
            (vec![(1, 0), (1, 1), (0, 1), (0, 2)], Color::YELLOW),
            (vec![(0, 0), (0, 1), (1, 0), (1, 1)], Color::GREEN),
            (vec![(1, 0), (0, 1), (1, 1), (2, 1)], Color::BLUE),
            (vec![(0, 0), (0, 1), (0, 2), (0, 3)], Color::GRAY),
            (vec![(0, 0), (1, 0), (2, 0), (0, 1)], Color::ORANGE),
            (vec![(0, 0), (1, 0), (2, 0), (2, 1)], Color::VIOLET),
        ]
    };
*/

pub trait Input {
    fn move_left(&self) -> bool;
    fn move_right(&self) -> bool;
    fn rotate_cw(&self) -> bool;
    fn rotate_ccw(&self) -> bool;
    fn fast_drop(&self) -> bool;
    fn instant_drop(&self) -> bool;
}

pub type Id = NonZeroU32;
pub type Position = (usize, usize);

pub fn add_positions(a: Position, b: Position) -> Position {
    (a.0 + b.0, a.1 + b.1)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    Plank,
}

pub enum TickChange {
    /// Active block is locked to the board.
    BlockLocked,
    /// New active block has arrived.
    NewBlock,
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
        let p0 = Point {
            id: gen_id(),
            origin_block_type: block_type,
        };

        let mut points_pos = HashMap::new();
        points_pos.insert(p0.id, (0, 0));

        let points = vec![p0];

        Self {
            id,
            block_type,
            points,
            points_pos,
        }
    }

    fn bounding_rect(&self) -> (usize, usize) {
        let mut max_x = 0;
        let mut max_y = 0;
        for &point_pos in self.points_pos.values() {
            max_x = max(max_x, point_pos.0);
            max_y = max(max_y, point_pos.1);
        }
        (max_x, max_y)
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }

    pub fn get_point_position(&self, point_id: Id) -> Option<Position> {
        self.points_pos.get(&point_id).copied()
    }
}

struct Timer {
    tick: u32,
}

impl Timer {
    fn new() -> Self {
        Self { tick: 0 }
    }

    fn tick(&mut self) {
        self.tick += 1;
    }

    fn restart(&mut self) {
        self.tick = 0;
    }

    fn has_elapsed(&self, duration: u32) -> bool {
        self.tick >= duration
    }

    fn tick_and_restart_if_elapsed(&mut self, duration: u32) -> bool {
        self.tick();
        if self.has_elapsed(duration) {
            self.restart();
            true
        } else {
            false
        }
    }
}

enum RepeatedInputState {
    Inactive,
    Wait,
    Repeat,
}

struct RepeatedInput {
    state: RepeatedInputState,
    timer: Timer,
    active: bool,
    wait_duration: u32,
    repeat_duration: u32,
}

impl RepeatedInput {
    fn new(wait_duration: u32, repeat_duration: u32) -> Self {
        Self {
            state: RepeatedInputState::Inactive,
            timer: Timer::new(),
            active: false,
            wait_duration,
            repeat_duration,
        }
    }

    fn tick(&mut self, active: bool) {
        if !active {
            self.state = RepeatedInputState::Inactive;
            self.active = false;
        } else {
            (self.state, self.active) = match self.state {
                RepeatedInputState::Inactive => {
                    self.timer.restart();
                    (RepeatedInputState::Wait, true)
                }
                RepeatedInputState::Wait => {
                    if self.timer.tick_and_restart_if_elapsed(self.wait_duration) {
                        (RepeatedInputState::Repeat, true)
                    } else {
                        (RepeatedInputState::Wait, false)
                    }
                }
                RepeatedInputState::Repeat => {
                    let active = self.timer.tick_and_restart_if_elapsed(self.repeat_duration);
                    (RepeatedInputState::Repeat, active)
                }
            };
        }
    }

    fn active(&self) -> bool {
        self.active
    }
}

struct SmartInput {
    move_left: RepeatedInput,
    move_right: RepeatedInput,
    rotate_cw: RepeatedInput,
    rotate_ccw: RepeatedInput,
    fast_drop: bool,
    instant_drop: bool,
}

impl SmartInput {
    fn new() -> Self {
        Self {
            move_left: RepeatedInput::new(WAIT_DURATION, REPEAT_DURATION),
            move_right: RepeatedInput::new(WAIT_DURATION, REPEAT_DURATION),
            rotate_cw: RepeatedInput::new(WAIT_DURATION, REPEAT_DURATION),
            rotate_ccw: RepeatedInput::new(WAIT_DURATION, REPEAT_DURATION),
            fast_drop: false,
            instant_drop: false,
        }
    }

    fn tick(&mut self, input: &dyn Input) {
        self.move_left.tick(input.move_left());
        self.move_right.tick(input.move_right());
        self.rotate_cw.tick(input.rotate_cw());
        self.rotate_ccw.tick(input.rotate_ccw());
        self.fast_drop = input.fast_drop();
        self.instant_drop = input.instant_drop();
    }
}

impl Input for SmartInput {
    fn move_left(&self) -> bool {
        self.move_left.active()
    }

    fn move_right(&self) -> bool {
        self.move_right.active()
    }

    fn rotate_cw(&self) -> bool {
        self.rotate_cw.active()
    }

    fn rotate_ccw(&self) -> bool {
        self.rotate_ccw.active()
    }

    fn fast_drop(&self) -> bool {
        self.fast_drop
    }

    fn instant_drop(&self) -> bool {
        self.instant_drop
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
        let active_block = Block::new(gen_id(), BlockType::Plank, &mut gen_id);
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
        let block = &self.active_block;
        let mut block_pos = self.active_block_pos;
        let block_rect = self.active_block.bounding_rect();
        self.input.tick(input);

        if self.input.move_left() {
            if block_pos.0 > 0 && !self.is_block_collides(block, (block_pos.0 - 1, block_pos.1)) {
                block_pos.0 -= 1;
            }
        }
        if self.input.move_right() {
            if block_pos.0 + block_rect.0 < BOARD_WIDTH - 1
                && !self.is_block_collides(block, (block_pos.0 + 1, block_pos.1))
            {
                block_pos.0 += 1;
            }
        }

        let drop_speed = if self.input.fast_drop() {
            self.rules.fast_drop_speed()
        } else {
            self.rules.drop_speed()
        };

        if self.drop_timer.tick_and_restart_if_elapsed(drop_speed) {
            if block_pos.1 + block_rect.1 == BOARD_HEIGHT - 1
                || self.is_block_collides(block, (block_pos.0, block_pos.1 + 1))
            {
                self.lock_active_block_to_board(block_pos);
                changes.push(TickChange::BlockLocked);

                //todo: remove filled rows

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

    /// Returns `true` if block would collide with any of board points.
    fn is_block_collides(&self, block: &Block, block_pos: Position) -> bool {
        for point in block.points() {
            let (x, y) = add_positions(block_pos, block.get_point_position(point.id).unwrap());
            if self.board[y][x].is_some() {
                return true;
            }
        }
        false
    }

    fn spawn_block(&mut self) {
        //todo: choose random block
        self.active_block = Block::new((self.gen_id)(), BlockType::Plank, &mut self.gen_id);
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
}

struct IdGenerator {
    next_id: Id,
}

impl IdGenerator {
    fn new() -> Self {
        Self {
            next_id: Id::new(1).unwrap(),
        }
    }
}

impl FnOnce<()> for IdGenerator {
    type Output = Id;

    extern "rust-call" fn call_once(self, _args: ()) -> Self::Output {
        unreachable!()
    }
}

impl FnMut<()> for IdGenerator {
    extern "rust-call" fn call_mut(&mut self, _args: ()) -> Self::Output {
        let id = self.next_id;
        self.next_id = Id::new(id.get() + 1).unwrap();
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_ids() {
        let mut gen_id = IdGenerator::new();
        assert_eq!(1, gen_id().get());
        assert_eq!(2, gen_id().get());
        assert_eq!(3, gen_id().get());
    }
}
