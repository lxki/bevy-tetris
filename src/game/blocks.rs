use bevy::render::color::Color;
use lazy_static::lazy_static;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

use super::Position;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum BlockType {
    I = 0,
    J,
    L,
    O,
    S,
    T,
    Z,
}

struct BlockInfo {
    points: Vec<Position>,
    color: Color,
}

impl BlockInfo {
    fn new(points: Vec<Position>, color: Color) -> Self {
        Self { points, color }
    }
}

lazy_static! {
    static ref BLOCKS: HashMap<BlockType, BlockInfo> = HashMap::from([
        (
            BlockType::I,
            BlockInfo::new(vec![(0, 0), (0, 1), (0, 2), (0, 3)], Color::CYAN)
        ),
        (
            BlockType::J,
            BlockInfo::new(vec![(0, 0), (0, 1), (1, 1), (2, 1)], Color::BLUE)
        ),
        (
            BlockType::L,
            BlockInfo::new(vec![(0, 1), (1, 1), (2, 1), (2, 0)], Color::ORANGE)
        ),
        (
            BlockType::O,
            BlockInfo::new(vec![(0, 0), (1, 0), (1, 1), (0, 1)], Color::YELLOW)
        ),
        (
            BlockType::S,
            BlockInfo::new(vec![(0, 1), (1, 1), (1, 0), (2, 0)], Color::GREEN)
        ),
        (
            BlockType::T,
            BlockInfo::new(vec![(0, 1), (1, 1), (1, 0), (2, 1)], Color::PURPLE)
        ),
        (
            BlockType::Z,
            BlockInfo::new(vec![(0, 0), (1, 0), (1, 1), (2, 1)], Color::RED)
        ),
    ]);
}

pub fn get_block_points(block_type: BlockType) -> &'static Vec<Position> {
    &BLOCKS[&block_type].points
}

pub fn get_block_color(block_type: BlockType) -> Color {
    BLOCKS[&block_type].color
}

pub fn get_random_block() -> BlockType {
    let block_count = BLOCKS.len();

    let mut rng = thread_rng();
    let block_i = rng.gen_range(0..block_count);
    unsafe { std::mem::transmute(block_i) }
}
