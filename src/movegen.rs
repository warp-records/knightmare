use std::cmp::{max, min};

use crate::magic::print_bitboard;

// bitboard movegen

const right_down_diag: u64 = 0x8040201008040201;
const right_up_diag: u64 = 0x102040810204080;

pub const COLUMN_LEFT: u64 = 0x8080808080808080;
pub const COLUMN_RIGHT: u64 = 0x8080808080808080 >> 7;
pub const ROW_TOP: u64 = 0xFF00000000000000;
pub const ROW_BOTTOM: u64 = 0x00000000000000FF;

const vertical_zeros_right: u64 = 0xFEFEFEFEFEFEFEFE;
const vertical_zeros_left: u64 = 0x7F7F7F7F7F7F7F7F;

fn shr(val: u64, dist: i8) -> u64 {
    if dist >= 0 {
        val >> dist
    } else {
        val << -dist
    }
}

fn get_diagonal_rays(x: u8, y: u8) -> (u64, u64) {
    let piece_bb = coords_to_bb(x, y);
    // necessary for underflow
    let x = x as i8;
    let y = y as i8;

    let mut right_down: u64 = 0;
    let mut right_up: u64 = 0;

    // shift right up diagonal to the line x-y
    right_down = shr(right_down_diag, x - y);

    // clip residue bits
    for i in 0..(x - y).abs() {
        let shift_amt = if x >= y { i } else { -i };
        let mask = if x >= y {
            vertical_zeros_left
        } else {
            vertical_zeros_right
        };
        right_down &= shr(mask, shift_amt);
    }

    right_up = shr(right_up_diag, x + y - 7);

    for i in 0..(x + y - 7).abs() {
        let shift_amt = if x + y - 7 <= 0 { -i } else { i };
        let mask = if x + y - 7 <= 0 {
            vertical_zeros_right
        } else {
            vertical_zeros_left
        };
        right_up &= shr(mask, shift_amt);
    }

    (right_down & !piece_bb, right_up & !piece_bb)
}

/// generate bitboard of unbounded diagonal ray starting at position
pub fn gen_diagonal_ray(x: u8, y: u8) -> u64 {
    let (right_down, right_up) = get_diagonal_rays(x, y);
    (right_down | right_up) & !coords_to_bb(x, y)
}

/// generate diagonal bitboard accounting for blockers, inclusive of blockers in ray and exclusve of origin
pub fn gen_blocked_diagonal(x: u8, y: u8, other_pieces: u64) -> u64 {
    let (right_down, right_up) = get_diagonal_rays(x, y);

    // partition board into areas relative to piece origin
    let top_area = u64::MAX << (7 - y) * 8;
    let bottom_area = !top_area;
    // pretty sure this will work
    let right_area = u64::wrapping_mul((1u64 << 8 - x) - 1, COLUMN_LEFT) & !COLUMN_LEFT;
    // bottom row is lost during first multiply
    let right_area = right_area | right_area >> 8;
    let left_area = !right_area;

    // parse blockers and generate blocked rays by quadrant
    let quad1 = right_area & top_area;
    let quad1_blockers: u64 = right_up & quad1 & other_pieces;
    let nearest = quad1_blockers.trailing_zeros();
    let quad1_diag = right_up & (u64::MAX >> 64 - min(nearest + 1, 64)) & quad1;

    let quad2 = left_area & top_area;
    let quad2_blockers: u64 = right_down & quad2 & other_pieces;
    let nearest = quad2_blockers.trailing_zeros();
    let quad2_diag = right_down & (u64::MAX >> 64 - min(nearest + 1, 64)) & quad2;

    let quad3 = left_area & bottom_area;
    let quad3_blockers: u64 = right_up & quad3 & other_pieces;
    let nearest = quad3_blockers.leading_zeros();
    let quad3_diag = right_up & (u64::MAX << 64 - min(nearest + 1, 64)) & quad3;

    let quad4 = right_area & bottom_area;
    let quad4_blockers: u64 = right_down & quad4 & other_pieces;
    let nearest = quad4_blockers.leading_zeros();
    let quad4_diag = right_down & (u64::MAX << 64 - min(nearest + 1, 64)) & quad4;

    quad1_diag | quad2_diag | quad3_diag | quad4_diag
}

/// generate bitboard of straight ray accounting for blockers, inclusive of blockers and exclusve of origin
/// returns (vertical, horizontal)
pub fn gen_straight_rays(x: u8, y: u8) -> (u64, u64) {
    let piece_bb = coords_to_bb(x, y);
    let x = x as i8;
    let y = y as i8;

    let vert = shr(COLUMN_LEFT, x);
    let horiz = shr(ROW_TOP, y * 8);

    (vert & !piece_bb, horiz & !piece_bb)
}

/// generate bitboard of unbounded straight ray starting at position
pub fn gen_straight_ray(x: u8, y: u8) -> u64 {
    let (col, row) = gen_straight_rays(x, y);
    col | row
}

pub fn gen_blocked_straight(x: u8, y: u8, other_pieces: u64) -> u64 {
    let (col, row) = gen_straight_rays(x, y);

    let top_area = u64::MAX << (7 - y) * 8;
    let bottom_area = !top_area;
    let top_area = top_area << 8;
    let right_area = u64::wrapping_mul((1u64 << 8 - x) - 1, COLUMN_LEFT) & !COLUMN_LEFT;
    let right_area = right_area | right_area >> 8;
    let left_area = !right_area << 1 & vertical_zeros_right;

    let nearest = (other_pieces & top_area).trailing_zeros();
    let mut top_ray = (u64::MAX >> 64 - min(nearest + 1, 64)) & col & top_area;
    if top_ray == 0 {
        top_ray = col & top_area;
    }

    let nearest = (other_pieces & bottom_area).trailing_zeros();
    let mut bottom_ray = (u64::MAX << min(nearest - 1, 64)) & col & bottom_area;
    if bottom_ray == 0 {
        bottom_ray = col & bottom_area;
    }

    let nearest = (other_pieces & left_area).trailing_zeros();
    let mut left_ray = (u64::MAX >> 64 - min(nearest + 1, 64)) & row & left_area;
    if left_ray == 0 {
        left_ray = row & left_area;
    }

    let nearest = (other_pieces & right_area).leading_zeros();
    let mut right_ray = (u64::MAX << 64 - min(nearest + 1, 64)) & row & right_area;
    if right_ray == 0 {
        right_ray = row & right_area;
    }

    top_ray | bottom_ray | left_ray | right_ray
}

pub fn gen_knight(x: u8, y: u8) -> u64 {
    // centered at 2, 2
    const knight_moves: u64 = 0x5088008850000000;
    let x = x as i8;
    let y = y as i8;

    let mut moves: u64 = shr(knight_moves, (x - 2) + (y - 2) * 8);
    if x < 2 {
        moves &= vertical_zeros_right << 0;
        moves &= vertical_zeros_right << 1;
    } else if x > 5 {
        moves &= vertical_zeros_left >> 0;
        moves &= vertical_zeros_left >> 1;
    }

    moves
}


/// convert a number of tiles - from left to right, bottom to top - to an (x, y) coordinate position
pub fn right_shift_to_coords(offset: u8) -> (u8, u8) {
    (offset % 8, 7 - (offset / 8))
}

// convert coordinates to right shift
pub fn coords_to_right_shift(x: u8, y: u8) -> u8 {
    x + (7-y) * 8
}

pub fn coords_to_left_shift(x: u8, y: u8) -> u8 {
    y * 8 - x + 7
}

/// generate a bitboard with a single tile at the given position.
pub fn coords_to_bb(x: u8, y: u8) -> u64 {
    0b1u64 << 63 - (x + y * 8)
}

/// right shift to bitboard cause I ain't typin this shit out every time
pub fn rs_to_bb(right_shift: u32) -> u64 {
    0b1000000000000000000000000000000000000000000000000000000000000000u64 >> right_shift
}
