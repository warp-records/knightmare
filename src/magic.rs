
use std::cmp::{min, max};
use arrayvec::*;

const right_down_diag: u64 = 0x8040201008040201;
const right_up_diag: u64 =    0x102040810204080;

const vertical_zeros_right: u64 = 0xFEFEFEFEFEFEFEFE;
const vertical_zeros_left: u64 = 0x7F7F7F7F7F7F7F7F;

fn shr(val: u64, dist: i8) -> u64 {
    if dist >= 0 {
        val >> dist
    } else {
        val << -dist
    }
}

// (right_down, right_up) for internal use
fn get_diagonal_rays(x: u8, y: u8) -> (u64, u64) {
    // necessary for underflow
    let x = x as i8;
    let y = y as i8;

    let mut right_down: u64 = 0;
    let mut right_up: u64 = 0;

    // shift right up diagonal to the line x-y
    right_down = shr(right_down_diag, x-y);

    // clip residue bits
    for i in 0..(x-y).abs() {
        let shift_amt = if x >= y { i } else { -i };
        let mask = if x >= y { vertical_zeros_left } else { vertical_zeros_right };
        right_down &= shr(mask, shift_amt);
    }

    right_up = shr(right_up_diag, x+y-7);

    for i in 0..(x+y-7).abs() {
        let shift_amt = if x+y-7 <= 0 { -i } else { i };
        let mask = if x+y-7 <= 0 { vertical_zeros_right } else { vertical_zeros_left };
        right_up &= shr(mask, shift_amt);
    }

    (right_down,  right_up)
}

pub fn gen_diagonal_ray(x: u8, y: u8) -> u64 {
    let (right_down, right_up) = get_diagonal_rays(x, y);
    right_down | right_up
}

/// generate diagonal bitboard accounting for blockers. Board is inclusive of blockers in ray
pub fn gen_clipped_diagonal(x: u8, y: u8, other_pieces: u64) -> u64 {
    let (right_down, right_up) = get_diagonal_rays(x, y);

    let top_area = u64::MAX << (7-y)*8;
    let bottom_area = !top_area;
    // pretty sure this will work
    let right_area = u64::wrapping_mul((1u64 << 8-x) - 1, column_left) & !column_left;
    // bottom row is lost during first multiply
    let right_area = right_area | right_area >> 8;
    let left_area = !right_area;

    // parse blockers and generate blocked rays by quadrant
    let quad1 = right_area & top_area;
    let quad1_blockers: u64 = right_up & quad1 & other_pieces;
    let nearest = quad1_blockers.trailing_zeros();
    let quad1_diag = right_up & (u64::MAX >> 64-min(nearest+1, 64)) & quad1;

    let quad2 = left_area & top_area;
    let quad2_blockers: u64 = right_down & quad2 & other_pieces;
    let nearest = quad2_blockers.trailing_zeros();
    let quad2_diag = right_down & (u64::MAX >> 64-min(nearest+1, 64)) & quad2;

    let quad3 = left_area & bottom_area;
    let quad3_blockers: u64 = right_up & quad3 & other_pieces;
    let nearest = quad3_blockers.leading_zeros();
    let quad3_diag = right_up & (u64::MAX << 64-min(nearest+1, 64)) & quad3;

    let quad4 = right_area & bottom_area;
    let quad4_blockers: u64 = right_down & quad4 & other_pieces;
    let nearest = quad4_blockers.leading_zeros();
    let quad4_diag = right_down & (u64::MAX << 64-min(nearest+1, 64)) & quad4;

    quad1_diag | quad2_diag | quad3_diag | quad4_diag
}

const column_left: u64 = 0x8080808080808080;
const row_top: u64 = 0xFF00000000000000;

fn gen_straight_rays(x: u8, y: u8) -> (u64, u64) {
    let x = x as i8;
    let y = y as i8;

    let col = shr(column_left, x);
    let row = shr(row_top, y*8);

    (col, row)
}

pub fn gen_straight_ray(x: u8, y: u8) -> u64 {
    let (col, row) = gen_straight_rays(x, y);
    col | row
}

pub fn gen_clipped_straight(x: u8, y: u8, other_pieces: u64) -> u64 {
    let (col, row) = gen_straight_rays(x, y);
    let piece_bit = 1u64 << coords_to_left_shift(x, y);

    let top_area = u64::MAX << (7-y)*8;
    let bottom_area = !top_area;
    let top_area = top_area << 8;
    let right_area = u64::wrapping_mul((1u64 << 8-x) - 1, column_left) & !column_left;
    let right_area = right_area | right_area >> 8;
    let left_area = !right_area << 1 & vertical_zeros_right;

    let nearest = (other_pieces & top_area).trailing_zeros();
    let mut top_ray = (u64::MAX >> 64-min(nearest+1, 63)) & col & top_area;
    if top_ray == 0 { top_ray = col & top_area; }

    let nearest = (other_pieces & bottom_area).trailing_zeros();
    let mut bottom_ray = (u64::MAX << min(nearest-1, 63)) & col & bottom_area;
    if bottom_ray == 0 { bottom_ray = col & bottom_area; }

    let nearest = (other_pieces & left_area).trailing_zeros();
    let mut left_ray = (u64::MAX >> 64-min(nearest+1, 63)) & row & left_area;
    if  left_ray == 0 { left_ray = row & left_area; }

    let nearest = (other_pieces & right_area).leading_zeros();
    let mut right_ray = (u64::MAX >> min(nearest+1, 63)) & row & right_area;
    if right_ray == 0 { right_ray = row & right_area; }

    top_ray | bottom_ray | left_ray | right_ray | piece_bit
}

pub fn gen_knight(x: u8, y: u8) -> u64 {
    // centered at 2, 2
    const knight_moves: u64 = 0x5088008850000000;
    let x = x as i8;
    let y = y as i8;

    let mut moves: u64 = shr(knight_moves, (x-2) + (y-2)*8);
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
pub fn shift_to_coords(offset: u8) -> (u8, u8) {
    (offset % 8, offset / 8)
}

// convert coordinates to right shift
pub fn coords_to_right_shift(x: u8, y: u8) -> u8 {
    x + y*8
}

pub fn coords_to_left_shift(x: u8, y: u8) -> u8 {
    (7-y)*8 + 7 - x
}

// attempt to generate a table of magic bitboards
// N is either 10, 11, or 12 depending on
struct MagicTable<const N: usize> {
    table: [u64; N],
    magic: u64,
}

pub fn gen_magic_table(pos: (u8, u8), orthogonal: bool) -> ArrayVec<u64, 4096> {
    let (x, y) = pos;
    let mut range_board = if orthogonal { gen_straight_ray(x, y) } else { gen_diagonal_ray(x, y) };

    // remove redundant ranks and files
    if x != 0 {
        range_board &= !column_left;
    }
    if x != 7 {
        range_board &= !(column_left >> 7);
    }
    if y != 0 {
        range_board &= !row_top;
    }
    if y != 7 {
        range_board &= !(row_top >> 56);
    }

    let mut table_sz: usize = 12;
    if x == 0 || x == 7 {
        table_sz -= 1;
    }
    if y == 0 || y == 7 {
        table_sz -= 1;
    }

    // store positions of each bit from the range board
    // which are going to be toggling
    let mut bit_positions = ArrayVec::<u32, 12>::new();
    while range_board != 0 {
        let next_pos = range_board.trailing_zeros();
        bit_positions.push(next_pos);
        range_board &= !(1u64 << next_pos);
    }

    let mut blocker_map: ArrayVec<Option<u64>, 4096> = ArrayVec::from([None; 4096]);
    unsafe {
        let max_len = blocker_map.len();
        // 1024, 2048, or 4096 permutations of rays
        blocker_map.set_len(2usize.pow(table_sz as u32));
        assert!(blocker_map.len() <= max_len);
    }
    let mut magic: u64 = 0;

    while magic <= u64::MAX {
        let mut found_magic = true;
        blocker_map = ArrayVec::new();

        // iteraete over every bitstring up to N bits
        for bitstr in 0..blocker_map.len() as u64 {
            // generate permutation of blockers along ray using current bitstring
            let mut blocker_board: u64 = 0;
            for tbl_idx in 0..table_sz {
                let nth_bit  = (bitstr & (1u64 << tbl_idx)) >> tbl_idx;
                blocker_board ^= nth_bit << (bit_positions[tbl_idx]);
            }

            // check for collision at the index our magic gives us
            let map_index = (blocker_board * magic) as usize % blocker_map.len();
            if blocker_map[map_index].is_some() {
                found_magic = false;
                break;
            } else {
                // populate blocker map with our current configuration
                blocker_map[map_index] = Some(todo!());
            }
        }

        if found_magic {
            break;
        }

        magic += 1;
    }

    ArrayVec::new()
}

pub fn print_bitboard(bb: u64) {
    print!("  ");
    for i in 0..8 {
        print!("{i} ");
    }
    println!();
    for rank in (0..8).rev() {
        print!("{} ", 7-rank);
        for file in (0..8).rev() {
            let square = rank * 8 + file;
            if (bb >> square) & 1 == 1 {
                print!("◼ ");
            } else {
                print!("◻ ");
            }
        }
        println!();
    }

    println!();
}
