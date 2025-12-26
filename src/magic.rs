
use crate::movegen::*;
use std::cmp::{min, max};
use arrayvec::*;

// attempt to generate a table of magic bitboards
// N is either 10, 11, or 12 depending on
struct MagicTable<const N: usize> {
    table: [u64; N],
    magic: u64,
}

pub fn gen_magic_table(x: u8, y: u8, orthogonal: bool) -> ArrayVec<u64, 4096> {
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
                let blocked_ray = if orthogonal {
                    gen_blocked_straight(x, y, blocker_board)
                } else {
                    gen_blocked_diagonal(x, y, blocker_board)
                };

                blocker_map[map_index] = Some(blocked_ray);
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
