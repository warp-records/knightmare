use arrayvec::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

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

#[derive(Debug)]
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

    magics_straight: Option<[[MagicTable; 8]; 8]>,
    magics_diagonal: Option<[[MagicTable; 8]; 8]>,

    kings: u64,
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
    // mask of threat lines of all enemy pieces generated at the beginning of each turn
    // idk if I need this actually
    // threats: u64,
}

#[derive(Debug)]
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

#[derive(Serialize, Deserialize)]
struct MagicsDb {
    magics_straight: [[MagicTable; 8]; 8],
    magics_diagonal: [[MagicTable; 8]; 8],
}
impl GameState {

    const MAGICS_FILE_PATH: &str = "magics/magics.db";

    pub fn new() -> GameState {
        const KINGS_INIT: u64 = chessboard!(
            0b_00001000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00001000
        );

        const QUEENS_INIT: u64 = chessboard!(
            0b_00010000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00010000
        );

        const ROOKS_INIT: u64 = chessboard!(
            0b_10000001
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_10000001
        );

        const KNIGHTS_INIT: u64 = chessboard!(
            0b_01000010
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_01000010
        );

        const BISHOPS_INIT: u64 = chessboard!(
            0b_00101000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00100100
        );

        const PAWNS_INIT: u64 = chessboard!(
            0b_00000000
            0b_11111111
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_11111111
            0b_00000000
        );

        const BLACK_SIDE: u64 = chessboard!(
            0b_11111111
            0b_11111111
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
        );

        const WHITE_SIDE: u64 = chessboard!(
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_11111111
            0b_11111111
        );

        GameState {
            turn: Color::White,
            magics_diagonal: None,
            magics_straight: None,
            kings:  KINGS_INIT,
            queens: QUEENS_INIT,
            rooks: ROOKS_INIT,
            knights: KNIGHTS_INIT,
            bishops: BISHOPS_INIT,
            pawns: PAWNS_INIT,
            en_passe: 0,
            short_castle: true,
            long_castle: true,
            black: BLACK_SIDE,
            white: WHITE_SIDE,
        }
    }


    pub fn init_magics(&mut self) {
        if let Ok(magics_db) = Self::read_magics_db() {
            self.magics_straight = Some(magics_db.magics_straight);
            self.magics_diagonal = Some(magics_db.magics_diagonal);
        } else {
            let mut magics_diagonal: [[MagicTable; 8]; 8] = Default::default();
            let mut magics_straight: [[MagicTable; 8]; 8] = Default::default();

            for x in 0..8 {
                for y in 0..8 {
                    magics_straight[x][y] = MagicTable::gen_table(x, y, true);
                    magics_diagonal[x][y] = MagicTable::gen_table(x, y, false);
                }
            }

            let file = File::create(Self::MAGICS_FILE_PATH).expect("Can't open file... what the fuck");
            let mut writer = BufWriter::new(file);

            let magics_db = MagicsDb {
                magics_straight: magics_straight,
                magics_diagonal: magics_diagonal,
            };

            bincode::serialize_into(&mut writer, &magics_db).expect("Failed to serialize and write magics db");

            self.magics_straight = Some(magics_db.magics_straight);
            self.magics_diagonal = Some(magics_db.magics_diagonal);
        }
    }

    fn read_magics_db() -> Result<MagicsDb, ()> {
        let mut file = BufReader::new(File::open(Self::MAGICS_FILE_PATH).map_err(|_| ())?);
        let magics_db: MagicsDb = bincode::deserialize_from(&mut file).map_err(|_| ())?;

        Ok(magics_db)
    }

    // Return order is from top to bottom, left to right formatted as (original_pos_bitboard, new_pos_bitboard)
    // implement pawn promotion later

    fn self_bb(&self) -> u64 {
        if self.turn == Color::White { self.white } else { self.black }
    }

    fn enemy_bb(&self) -> u64 {
        if self.turn == Color::White { self.black } else { self.white }
    }
    fn self_bb_mut(&mut self) -> &mut u64 {
        if self.turn == Color::White { &mut self.white } else { &mut self.black }
    }

    fn enemy_bb_mut(&mut self) -> &mut u64 {
        if self.turn == Color::White { &mut self.black } else { &mut self.white }
    }

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

    // let short_castle_ray: u64 = 0b00000110;
    // let long_castle_ray: u64 = 0b01110000;
    // if self.turn == Color::Black {
    //     short_castle_ray <<= 8*7;
    //     long_castle_ray <<= 8*7;
    // }

    // if self.short_castle && short_castle_ray & whole_board == 0 {

    // }

    // ignore whether or not we're in check for now
    // castling moves implemented in king moves
    pub fn rook_moves(&self) -> ArrayVec<Move, 14> {
        let mut moveset = ArrayVec::new();

        let self_bb = self.self_bb();

        let mut rooks_bb = self.self_bb() & self.rooks;

        for _ in 0..2 {
            let shift = rooks_bb.leading_zeros();
            let src = shift_to_coords(shift as u8);

            if shift == 64 {
                break;
            }

            rooks_bb ^= rs_to_bb(shift);

            let blockers = self.black | self.white & !coords_to_bb(src.0, src.1);
            let magics_straight = self.magics_straight.as_ref().unwrap();
            let mut moves_bb = magics_straight[src.0 as usize][src.1 as usize].get_ray(blockers).unwrap();

            moves_bb &= !self_bb;

            while moves_bb != 0 {
                let shift = moves_bb.leading_zeros();
                let dest = shift_to_coords(shift as u8);
                moveset.push(Move::new(src, dest));

                moves_bb ^= rs_to_bb(shift);
            }

        }

        moveset
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn rook_moves() {
        let mut game = GameState::new();
        game.init_magics();

    }
}
