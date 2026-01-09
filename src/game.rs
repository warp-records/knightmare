use arrayvec::*;

use crate::{magic::MagicTable, movegen::*};

// thanks https://stackoverflow.com/questions/47582781/multi-line-integer-literals-in-rust
#[macro_export]
macro_rules! chessboard {
    ($line0:tt $line1:tt $line2:tt $line3:tt $line4:tt $line5:tt $line6:tt $line7:tt) => {
        ($line0 << 56)
            | ($line1 << 48)
            | ($line2 << 40)
            | ($line3 << 32)
            | ($line4 << 24)
            | ($line5 << 16)
            | ($line6 << 8)
            | ($line7 << 0)
    };
}

#[derive(PartialEq)]
enum Color {
    Black,
    White,
}

enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Horses,
    Pawn,
}

pub struct GameState {
    turn: Color,

    magics_straight: Box<[[MagicTable; 8]; 8]>,
    magics_diagonal: Box<[[MagicTable; 8]; 8]>,

    king: u64,
    queens: u64,
    rooks: u64,
    bishops: u64,
    knights: u64,
    pawns: u64,
    // bitboard representing any king move, en passe
    en_passe: u64,
    short_castle: bool,
    long_castle: bool,
    /// positions of all pieces of a given color
    black: u64,
    white: u64,
    /// mask of threat lines of all enemy pieces generated at the beginning of each turn
    threats: u64,
}

pub struct Move {
    src: (u8, u8),
    dest: (u8, u8),
    promo: Option<PieceType>,
    castle: bool,
}

impl Move {
    pub fn new(src: (u8, u8), dest: (u8, u8)) -> Self {
        Move {
            src: src,
            dest: dest,
            promo: None,
            castle: false,
        }
    }
}

impl GameState {

    // Return order is from top to bottom, left to right formatted as (original_pos_bitboard, new_pos_bitboard)
    // implement pawn promotion later

    // fn self_bb(&mut self) -> &mut u64 {
    //     if self.turn == Color::White { &mut self.white } else { &mut self.black }
    // }

    // fn enemy_bb(&mut self) -> &mut u64 {
    //     if self.turn == Color::White { &mut self.black } else { &mut self.white }
    // }

    // (moves list, threat lines)
    // pub fn king_moves(&self) -> ArrayVec<Move, 8> {
    //     let mut moves = ArrayVec::new();

    //     let king_bb = if self.turn == Color::Black {
    //         self.black.king
    //     } else {
    //         self.white.king
    //     };
    //     let opponent_threats = if self.turn == Color::Black {
    //         self.white.threats
    //     } else {
    //         self.black.threats
    //     };

    //     let mut move_mask: u64 = 0xE0A0E00000000000;
    //     let move_mask_pos = 9;
    //     let self_threat: u64 = if king_bb.leading_zeros() > move_mask_pos {
    //         move_mask >> king_bb.leading_zeros() - move_mask_pos
    //     } else {
    //         move_mask >> move_mask_pos - king_bb.leading_zeros()
    //     };

    //     // up left
    //     moves.push(king_bb << (8 + 1));
    //     // up
    //     moves.push(king_bb << 8);
    //     // up right
    //     moves.push(king_bb << (8 - 1));
    //     // etc
    //     moves.push(king_bb << 1);
    //     moves.push(king_bb >> 1);
    //     moves.push(king_bb >> (8 - 1));
    //     moves.push(king_bb >> 8);
    //     moves.push(king_bb >> (8 + 1));

    //     moves.retain(|bb| (*bb & opponent_threats == 0));
    //     (moves, self_threat)
    // }

    // ignore whether or not we're in check for now
    // pub fn rook_moves(&self) -> ArrayVec<Move, 14> {
    //     let mut moveset = ArrayVec::new();

    //     let self_bb = *self.self_bb();
    //     let enemy_bb = *self.self_bb();

    //     let mut rooks_bb = *self.self_bb() & self.rooks;

    //     for _ in 0..2 {
    //         let shift = rooks_bb.leading_zeros();
    //         let src = shift_to_coords(shift as u8);

    //         if shift == 64 {
    //             break;
    //         }

    //         let blockers = self.black | self.white & !coords_to_bb(src.0, src.1);
    //         let mut moves_bb = self.magics_straight[src.0 as usize][src.1 as usize].get_ray(blockers).unwrap();

    //         moves_bb &= !self_bb;

    //         while moves_bb != 0 {
    //             let shift = moves_bb.leading_zeros();
    //             let dest = shift_to_coords(shift as u8);
    //             moveset.push(Move::new(src, dest));

    //             moves_bb ^= rs_to_bb(shift);
    //         }
    //     }

    //     moveset
    // }
}
