use crate::movegen::*;
use arrayvec::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};

// attempt to generate a table of magic bitboards
#[derive(Default, Serialize, Deserialize)]
pub struct MagicTable {
    table: Vec<u64>,
    magic: u64,
    clipped_ray: u64,
    // either 10, 11, or 12
    index_bits: u8,
}

impl MagicTable {
    /// Given a set of blockers, return a ray of the movespan in that space.
    /// Type and origin of ray is computed while generating table.
    pub fn get_ray(&self, blocker_board: u64) -> Option<u64> {
        let clipped_blockers = self.clipped_ray & blocker_board;
        let ray = self.table[Self::gen_table_idx(clipped_blockers, self.magic, self.index_bits)];
        if ray != 0 {
            Some(ray)
        } else {
            None
        }
    }

    /// Generate an index into a magic bitboard table, assumes blocker_board is already trimmed
    fn gen_table_idx(blocker_board: u64, magic: u64, index_bits: u8) -> usize {
        (blocker_board.wrapping_mul(magic) >> (64 - index_bits)) as usize
    }

    /// remove edges from diagonal ray
    fn clip_diagonal(mut range_board: u64) -> u64 {
        range_board &= !COLUMN_LEFT;
        range_board &= !COLUMN_RIGHT;
        range_board &= !ROW_TOP;
        range_board &= !ROW_BOTTOM;

        range_board
    }

    // remove top and bottom edges from vertical ray and left and right edgs from horizontal ray
    fn clip_straight(vertical: u64, horizontal: u64) -> u64 {
        (vertical & !ROW_TOP & !ROW_BOTTOM) | (horizontal & !COLUMN_LEFT & !COLUMN_RIGHT)
    }

    fn calc_shift(x: u8, y: u8) -> u8 {
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
    pub fn gen_table(x: usize, y: usize, straight: bool) -> Self {
        let x = x as u8;
        let y = y as u8;

        let mut range_board = if straight {
            let (horiz, vert) = gen_straight_rays(x, y);
            Self::clip_straight(horiz, vert)
        } else {
            let diag = gen_diagonal_ray(x, y);
            Self::clip_diagonal(diag)
        };

        let clipped_ray = range_board;

        let index_bits = Self::calc_shift(x, y);

        // store positions of each bit from the range board
        // which are going to be toggling
        let mut bit_positions = ArrayVec::<u32, 12>::new();
        while range_board != 0 {
            let next_pos = range_board.trailing_zeros();
            bit_positions.push(next_pos);
            range_board &= !(1u64 << next_pos);
        }

        let mut blocker_map: Vec<u64> = vec![0; 4096];
        let mut blocker_map_occupied: Vec<bool> = vec![false; 4096];
        let max_len = 2usize.pow(index_bits as u32);
        unsafe {
            // 1024, 2048, or 4096 permutations of rays
            blocker_map.set_len(max_len);
            blocker_map_occupied.set_len(max_len);
        }
        assert!(blocker_map.len() <= max_len);

        let mut rng = StdRng::seed_from_u64(0);
        let mut magic: u64 = 0;

        loop {
            let mut found_magic = true;
            blocker_map_occupied.fill(false);
            magic = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>();

            // iteraete over every bitstring up to N bits
            for bitstr in 0..=blocker_map.len() as u64 {
                // generate permutation of blockers along ray using current bitstring
                let mut blocker_board: u64 = 0;
                for tbl_idx in 0..bit_positions.len() {
                    let nth_bit = (bitstr & (1u64 << tbl_idx)) >> tbl_idx;
                    blocker_board |= nth_bit << (bit_positions[tbl_idx]);
                }

                // check for collision at the index our magic gives us
                // let map_index = (blocker_board.wrapping_mul(magic) >> (64-index_bits)) as usize;
                let map_index = MagicTable::gen_table_idx(blocker_board, magic, index_bits);

                let blocked_ray = if straight {
                    let blocked_straight = gen_blocked_straight(x, y, blocker_board);
                    blocked_straight
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

        let blocker_map = blocker_map
            .into_iter()
            .zip(blocker_map_occupied)
            .map(|(val, occupied)| if occupied { val } else { 0 })
            .collect::<Vec<u64>>();

        Self {
            table: blocker_map,
            magic: magic,
            clipped_ray: clipped_ray,
            index_bits: index_bits,
        }
    }
}



pub fn print_bitboard(bb: u64) {
    print!("  ");
    for i in 0..8 {
        print!("{i} ");
    }
    println!();
    // Print from rank 8 (y=7) at top to rank 1 (y=0) at bottom
    for y in (0..8).rev() {
        print!("{} ", y);
        for file in (0..8).rev() {
            let square = y * 8 + file;
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
