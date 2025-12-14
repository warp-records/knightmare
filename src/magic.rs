
use std::cmp;

const right_down_diag: u64 = 0x8040201008040201;
const left_up_diag: u64 =    0x102040810204080;

const vertical_zeros_right: u64 = 0xFEFEFEFEFEFEFEFE;
const vertical_zeros_left: u64 = 0x7F7F7F7F7F7F7F7F;

fn shr(val: u64, dist: i8) -> u64 {
    if dist >= 0 {
        val >> dist
    } else {
        val << -dist
    }
}

pub fn gen_diagonal(x: u8, y: u8) -> u64 {
    // necessary for underflow
    let x = x as i8;
    let y = y as i8;

    let mut right_down: u64 = 0;
    let mut left_up: u64 = 0;

    // shift right up diagonal to the line x-y
    right_down = shr(right_down_diag, x-y);

    // clip residue bits
    for i in 0..(x-y).abs() {
        let shift_amt = if x >= y { i } else { -i };
        let mask = if x >= y { vertical_zeros_left } else { vertical_zeros_right };
        right_down &= shr(mask, shift_amt);
    }

    left_up = shr(left_up_diag, x+y-7);

    for i in 0..(x+y-7).abs() {
        let shift_amt = if x+y-7 <= 0 { -i } else { i };
        let mask = if x+y-7 <= 0 { vertical_zeros_right } else { vertical_zeros_left };
        left_up &= shr(mask, shift_amt);
    }


    right_down | left_up
}

const column_left: u64 = 0x8080808080808080;
const row_top: u64 = 0xFF00000000000000;

pub fn gen_straight(x: u8, y: u8) -> u64 {
    let x = x as i8;
    let y = y as i8;

    let col = shr(column_left, x);
    let row = shr(row_top, y*8);

    col | row
}

pub fn print_bitboard(bb: u64) {
    for rank in (0..8).rev() {
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
}
