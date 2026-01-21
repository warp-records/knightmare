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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    /// positions of all pieces of a given color
    black: u64,
    white: u64,
    // bitboard representing any king move, en passe
    en_passe: u64,
    short_castle: bool,
    long_castle: bool,
    // mask of threat lines of all enemy pieces generated at the beginning of each turn
    // idk if I need this actually
    // threats: u64,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
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
            0b_00100100
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
            black: BLACK_SIDE,
            white: WHITE_SIDE,
            en_passe: 0,
            short_castle: true,
            long_castle: true,
        }
    }

    // primarily used for tests
    // TODO: read castling,
    pub fn try_from_fen(fen: &str) -> Result<Self, ()> {

        let mut kings: u64 = 0;
        let mut queens: u64 = 0;
        let mut rooks: u64 = 0;
        let mut bishops: u64 = 0;
        let mut knights: u64 = 0;
        let mut pawns: u64 = 0;
        let mut black: u64 = 0;
        let mut white: u64 = 0;

        let mut row: u8 = 0;

        for row_contents in fen.split('/') {
            if row > 7 {
                return Err(())
            }
            let mut col: u8 = 0;

            for ch in row_contents.chars() {
                let color: &mut u64 = if ch.is_lowercase() { &mut black } else { &mut white };

                if col > 7 { return Err(()); }
                if let Some(space) = ch.to_digit(10) {
                    col += space as u8;
                    continue;
                }

                match ch.to_ascii_lowercase() {
                    'k' => {
                        kings |= coords_to_bb(col, row);
                    },
                    'q' => {
                        queens |= coords_to_bb(col, row);
                    },
                    'r' => {
                        rooks |= coords_to_bb(col, row);
                    },
                    'b' => {
                        bishops |= coords_to_bb(col, row);
                    },
                    'n' => {
                        knights |= coords_to_bb(col, row);
                    },
                    'p' => {
                        pawns |= coords_to_bb(col, row);
                    },
                    _ => {
                        return Err(())
                    }
                }

                *color |= coords_to_bb(col, row);
                col += 1;
            }

            if col != 8 { return Err(()); }

            row += 1;
        }

        if row != 8 { return Err(()) }

        Ok(GameState {
            turn: Color::White,
            magics_straight: None,
            magics_diagonal: None,
            kings,
            queens,
            rooks,
            bishops,
            knights,
            pawns,
            black,
            white,
            // handle proper initialization of these special fields later
            en_passe: 0u64,
            short_castle: true,
            long_castle: true,
        })
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
    pub fn rook_moves(&self) -> ArrayVec<Move, 28> {
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
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    pub fn import_fen() {
        let gamestate = GameState::try_from_fen("r1bk3r/p2pBpNp/n4n2/1p1NP2P/6P1/3P4/P1P1K3/q5b1");
        assert!(gamestate.is_ok());

        let gamestate = GameState::try_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        assert!(gamestate.is_ok());

        let gamestate = gamestate.unwrap();
        let normal_setup = GameState::new();
        assert!(gamestate.black == normal_setup.black);
        assert!(gamestate.white == normal_setup.white);
        assert!(gamestate.kings == normal_setup.kings);
        assert!(gamestate.queens == normal_setup.queens);
        assert!(gamestate.rooks == normal_setup.rooks);
        assert!(gamestate.bishops == normal_setup.bishops);
        assert!(gamestate.knights == normal_setup.knights);
        assert!(gamestate.pawns == normal_setup.pawns);


        let gamestate = GameState::try_from_fen("8/8/8/4p1K1/2k1P3/8/8/8");
        assert!(gamestate.is_ok());

        let gamestate = GameState::try_from_fen("8/5k2/3p4/1p1Pp2p/pP2Pp1P/P4P1K/8/8");
        assert!(gamestate.is_ok());

        let gamestate = GameState::try_from_fen("rnbqknr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");
        assert!(gamestate.is_err());

        let gamestate = GameState::try_from_fen("rnbqknr/pppppppp/9/8/8/8/PPPPPPPP/RNBQKBNR");
        assert!(gamestate.is_err());

        let gamestate = GameState::try_from_fen("rnbqknr/pppppppp/9/8/8/8/PPPPPPPPRNBQKBNR");
        assert!(gamestate.is_err());

        let gamestate = GameState::try_from_fen("rnbqknX/pppppppp/9/8/8/8/PPPPPPPPRNBQKBNR");
        assert!(gamestate.is_err());
    }

    #[test]
    pub fn rook_moves() {
        // starting board except with a rook at 4, 3
        let mut game = GameState::try_from_fen("rnbqkbnr/pppppppp/8/4R3/8/8/PPPPPPPP/RNBQKBN1").unwrap();
        game.init_magics();

        let mut expected = vec![
            Move::new((4, 3), (4, 1)),
            Move::new((4, 3), (4, 2)),
            Move::new((4, 3), (4, 4)),
            Move::new((4, 3), (4, 5)),
            Move::new((4, 3), (0, 3)),
            Move::new((4, 3), (1, 3)),
            Move::new((4, 3), (2, 3)),
            Move::new((4, 3), (3, 3)),
            Move::new((4, 3), (5, 3)),
            Move::new((4, 3), (6, 3)),
            Move::new((4, 3), (7, 3))
        ];
        expected.sort();

        let mut moves: Vec<Move> = game.rook_moves().to_vec();
        moves.sort();

        assert_eq!(moves, expected);

        // more complex position
        let mut game = GameState::try_from_fen("2r2r2/pk4pp/1p6/P1p1B3/8/2R2n2/2P2P1P/1R3K2").unwrap();
        game.init_magics();

        let mut expected = vec![
            // rook at (1, 7)
            Move::new((1, 7), (0, 7)),
            Move::new((1, 7), (2, 7)),
            Move::new((1, 7), (3, 7)),
            Move::new((1, 7), (4, 7)),
            Move::new((1, 7), (1, 6)),
            Move::new((1, 7), (1, 5)),
            Move::new((1, 7), (1, 4)),
            Move::new((1, 7), (1, 3)),
            Move::new((1, 7), (1, 2)),

            // rook at (2, 5)
            Move::new((2, 5), (0, 5)),
            Move::new((2, 5), (1, 5)),
            Move::new((2, 5), (3, 5)),
            Move::new((2, 5), (4, 5)),
            Move::new((2, 5), (5, 5)),
            Move::new((2, 5), (2, 4)),
            Move::new((2, 5), (2, 3)),
        ];
        expected.sort();

        let mut moves: Vec<Move> = game.rook_moves().to_vec();
        moves.sort();

        assert_eq!(moves, expected);
    }

    #[test]
    pub fn rook_moves_edge() {
        // Rook in corner a1 with minimal blocking
        // FEN: 8/8/8/8/8/8/1P6/R7 - white rook at a1, pawn at b2
        let mut game = GameState::try_from_fen("8/8/8/8/8/8/1P6/R7").unwrap();
        game.init_magics();

        let mut expected = vec![
            // Rook at (0, 7) - a1
            Move::new((0, 7), (1, 7)),  // right along rank
            Move::new((0, 7), (2, 7)),
            Move::new((0, 7), (3, 7)),
            Move::new((0, 7), (4, 7)),
            Move::new((0, 7), (5, 7)),
            Move::new((0, 7), (6, 7)),
            Move::new((0, 7), (7, 7)),
            Move::new((0, 7), (0, 6)),  // up along file
            Move::new((0, 7), (0, 5)),
            Move::new((0, 7), (0, 4)),
            Move::new((0, 7), (0, 3)),
            Move::new((0, 7), (0, 2)),
            Move::new((0, 7), (0, 1)),
            Move::new((0, 7), (0, 0)),
        ];
        expected.sort();

        let mut moves: Vec<Move> = game.rook_moves().to_vec();
        moves.sort();

        assert_eq!(moves, expected);
    }
}
