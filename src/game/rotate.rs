use std::collections::HashMap;

use super::{Block, Id, Position, BOARD_HEIGHT, BOARD_WIDTH};

pub fn rotate_block<F>(
    block: &Block,
    block_pos: Position,
    check_collision: F,
) -> Option<(HashMap<Id, Position>, Position)>
where
    F: Fn(&[Position], Position) -> bool,
{
    let (block_w, block_h) = (block.width(), block.height());
    let cx = (block_w / 2) as i32;
    let cy = (block_h / 2) as i32;

    let points = &block.points;
    let mut rot_points = Vec::with_capacity(block.points.len());
    for p in points {
        let point_pos = block.get_point_position(p.id).unwrap();
        let x = point_pos.0 as i32 - cx;
        let y = point_pos.1 as i32 - cy;

        let rot_p = (-y + cx, x + cy);
        rot_points.push(rot_p);
    }

    let min_x = rot_points.iter().map(|&(x, _)| x).min().unwrap();
    let min_y = rot_points.iter().map(|&(_, y)| y).min().unwrap();

    let rot_piece_pos = (block_pos.0 as i32 + min_x, block_pos.1 as i32 + min_y);
    if rot_piece_pos.0 < 0
        || rot_piece_pos.0 as usize + block_h >= BOARD_WIDTH
        || rot_piece_pos.1 < 0
        || rot_piece_pos.1 as usize + block_w >= BOARD_HEIGHT
    {
        return None;
    }

    let rot_block_pos = (rot_piece_pos.0 as usize, rot_piece_pos.1 as usize);
    let rot_points = rot_points
        .into_iter()
        .map(|(x, y)| ((x - min_x) as usize, (y - min_y) as usize))
        .collect::<Vec<_>>();

    if !check_collision(&rot_points, rot_block_pos) {
        return None;
    }

    let mut points_pos = HashMap::with_capacity(points.len());
    for (p, pos) in points.iter().zip(rot_points.iter()) {
        points_pos.insert(p.id, *pos);
    }
    Some((points_pos, rot_block_pos))
}
