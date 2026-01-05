
use crate::movegen::*;
use std::cmp::{min, max};
use arrayvec::*;
use rand::Rng;

// attempt to generate a table of magic bitboards
// N is either 10, 11, or 12 depending on
struct MagicTable<const N: usize> {
    table: [u64; N],
    magic: u64,
}

/// generate an index into a magic bitboard table, assumes blocker_board is already trimmed
pub fn gen_table_idx(blocker_board: u64, magic: u64, table_sz: usize) -> usize {
    (blocker_board.wrapping_mul(magic) >> (64-table_sz)) as usize
}

/// remove redundant ranks and files for magic bitboard lookup
/// returns (clipped_board, num_edges_clipped)
pub fn clip_range_board(x: u8, y: u8, mut range_board: u64) -> (u64, usize) {
    let mut removed_edges = 0;

    if x != 0 {
        range_board &= !column_left;
        removed_edges += 1;
    }
    if x != 7 {
        range_board &= !(column_left >> 7);
        removed_edges += 1;
    }
    if y != 0 {
        range_board &= !row_top;
        removed_edges += 1;
    }
    if y != 7 {
        range_board &= !(row_top >> 56);
        removed_edges += 1;
    }

    (range_board, removed_edges)
}

/// remove edges from diagonal ray
pub fn clip_diagonal(mut range_board: u64) -> u64 {

    range_board &= !column_left;
    range_board &= !column_right;
    range_board &= !row_top;
    range_board &= !row_bottom;

    range_board
}

// remove top and bottom edges from vertical ray and left and right edgs from horizontal ray
pub fn clip_straight(vertical: u64, horizontal: u64) -> u64 {
    (vertical & !row_top & !row_bottom) | (horizontal & !column_left & !column_right)
}

pub fn calc_shift(x: u8, y: u8) -> usize {
    let mut shift_amt = 10;

    if x == 0 || x == 7 {
        shift_amt += 1;
    }

    if y == 0 || y == 7 {
        shift_amt += 1;
    }

    shift_amt
}

/// generate a lookup table of bitboards representing the blocked movespace of a piece, and a magic value for indexing
pub fn gen_magic_table(x: u8, y: u8, orthogonal: bool) -> (ArrayVec<u64, 4096>, u64) {
    let mut range_board = if orthogonal {
        let (horiz, vert) = gen_straight_rays(x, y);
        clip_straight(horiz, vert)
    } else {
        let diag = gen_diagonal_ray(x, y);
        clip_diagonal(diag)
    };

    let table_sz = calc_shift(x, y);

    // store positions of each bit from the range board
    // which are going to be toggling
    let mut bit_positions = ArrayVec::<u32, 12>::new();
    while range_board != 0 {
        let next_pos = range_board.trailing_zeros();
        bit_positions.push(next_pos);
        range_board &= !(1u64 << next_pos);
    }

    let mut blocker_map: ArrayVec<u64, 4096> = ArrayVec::from([0; 4096]);
    let mut blocker_map_occupied: ArrayVec<bool, 4096> = ArrayVec::from([false; 4096]);
    let max_len = 2usize.pow(table_sz as u32);
    unsafe {
        // 1024, 2048, or 4096 permutations of rays
        blocker_map.set_len(max_len);
        blocker_map_occupied.set_len(max_len);
    }
    assert!(blocker_map.len() <= max_len);

    let mut rng = rand::thread_rng();
    let mut magic: u64 = 0;
    // let mut blocker_map: [u64; 4096] = [0u64; 4096];
    // let mut blocker_map_occupied: [bool; 4096] = [false; 4096];

    loop {
        let mut found_magic = true;
        blocker_map_occupied.fill(false);
        magic = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>();

        // iteraete over every bitstring up to N bits
        for bitstr in 0..blocker_map.len() as u64 {
            // generate permutation of blockers along ray using current bitstring
            let mut blocker_board: u64 = 0;
            for tbl_idx in 0..bit_positions.len() {
                let nth_bit  = (bitstr & (1u64 << tbl_idx)) >> tbl_idx;
                blocker_board |= nth_bit << (bit_positions[tbl_idx]);
            }

            // check for collision at the index our magic gives us
            // let map_index = (blocker_board.wrapping_mul(magic) >> (64-table_sz)) as usize;
            let map_index = gen_table_idx(blocker_board, magic, table_sz);

            let blocked_ray = if orthogonal {
                gen_blocked_straight(x, y, blocker_board)
            } else {
                gen_blocked_diagonal(x, y, blocker_board)
            };

            if blocker_map_occupied[map_index] {
                let existing_move = blocker_map[map_index];
                if existing_move != blocked_ray {
                    found_magic = false;
                    break;
                }
            } else {
                // populate blocker map with our current configuration
                blocker_map[map_index] = blocked_ray;
                blocker_map_occupied[map_index] = true;
            }
        }

        if found_magic {
            break;
        }

    }

    let blocker_map = blocker_map.into_iter()
        .zip(blocker_map_occupied)
        .map(|(val, occupied)| if occupied { val } else { 0 })
        .collect::<ArrayVec<u64, 4096>>();


    (blocker_map, magic)
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
