use arrayvec::*;
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::path::Path;
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

        // FEN is read top-to-bottom (rank 8 first), so row 0 = rank 8 = y=7
        let mut fen_row: u8 = 0;

        for row_contents in fen.split('/') {
            if fen_row > 7 {
                return Err(())
            }
            // Convert FEN row (0=rank8, 7=rank1) to y coordinate (0=rank1, 7=rank8)
            let y = 7 - fen_row;
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
                        kings |= coords_to_bb(col, y);
                    },
                    'q' => {
                        queens |= coords_to_bb(col, y);
                    },
                    'r' => {
                        rooks |= coords_to_bb(col, y);
                    },
                    'b' => {
                        bishops |= coords_to_bb(col, y);
                    },
                    'n' => {
                        knights |= coords_to_bb(col, y);
                    },
                    'p' => {
                        pawns |= coords_to_bb(col, y);
                    },
                    _ => {
                        return Err(())
                    }
                }

                *color |= coords_to_bb(col, y);
                col += 1;
            }

            if col != 8 { return Err(()); }

            fen_row += 1;
        }

        if fen_row != 8 { return Err(()) }

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

            let path = Path::new(Self::MAGICS_FILE_PATH);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("Can't create directory");
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
    /// use a bitboard as a
    pub fn moves_from_bb<const N: usize>(mut moves_bb: u64, src: (u8, u8)) -> ArrayVec<Move, N> {
        let mut moveset = ArrayVec::new();

        while moves_bb != 0 {
            let shift = moves_bb.leading_zeros();
            let dest = right_shift_to_coords(shift as u8);
            moveset.push(Move::new(src, dest));

            moves_bb ^= rs_to_bb(shift);
        }

        moveset
    }

    // ignore whether or not we're in check for now
    // castling moves implemented in king moves
    pub fn rook_moves(&self) -> ArrayVec<Move, 28> {
        let mut moveset = ArrayVec::new();

        let self_bb = self.self_bb();

        let mut rooks_bb = self.self_bb() & self.rooks;

        for _ in 0..2 {
            if rooks_bb == 0 { break; }
            let shift = rooks_bb.leading_zeros();
            let src = right_shift_to_coords(shift as u8);

            rooks_bb ^= rs_to_bb(shift);

            let blockers = self.black | self.white & !coords_to_bb(src.0, src.1);
            let magics_straight = self.magics_straight.as_ref().unwrap();
            let mut moves_bb = magics_straight[src.0 as usize][src.1 as usize].get_ray(blockers).unwrap();

            moves_bb &= !self_bb;

            let curr_moves = Self::moves_from_bb::<14>(moves_bb, src);
            moveset.extend(curr_moves);
        }

        moveset
    }

    pub fn bishop_moves(&self) -> ArrayVec<Move, 26> {
        let mut moveset = ArrayVec::new();

        let self_bb = self.self_bb();

        let mut bishops_bb = self.self_bb() & self.bishops;

        for _ in 0..2 {
            if bishops_bb == 0 { break; }

            let shift = bishops_bb.leading_zeros();
            let src = right_shift_to_coords(shift as u8);

            bishops_bb ^= rs_to_bb(shift);

            let blockers = self.black | self.white & !coords_to_bb(src.0, src.1);
            let magics_straight = self.magics_diagonal.as_ref().unwrap();
            let mut moves_bb = magics_straight[src.0 as usize][src.1 as usize].get_ray(blockers).unwrap();

            moves_bb &= !self_bb;
            let curr_moves = Self::moves_from_bb::<13>(moves_bb, src);
            moveset.extend(curr_moves);
        }

        moveset
    }

    pub fn knight_moves(&self) -> ArrayVec<Move, 16> {
        let mut moveset = ArrayVec::new();

        let self_bb = self.self_bb();
        let mut knights_bb = self.self_bb() & self.knights;

        for _ in 0..2 {
            if knights_bb == 0 { break; }
            let shift = knights_bb.leading_zeros();
            let src = right_shift_to_coords(shift as u8);

            knights_bb ^= rs_to_bb(shift);
            let mut moves_bb = gen_knight(src.0, src.1);
            moves_bb &= !self_bb;
            let curr_moves = Self::moves_from_bb::<8>(moves_bb, src);
            moveset.extend(curr_moves);
        }

        moveset
    }

    // I think there's 30 possible pawn moves??
    pub fn pawn_moves(&self) -> ArrayVec<Move, 30> {
        let moveset = ArrayVec::new();

        let self_bb = self.self_bb();
        let mut pawns_bb = self_bb & self.pawns;

        for _ in 0..8 {
            if pawns_bb == 0 { break; }

            let shift = self.pawns.leading_zeros();
            let src = right_shift_to_coords(shift as u8);

            pawns_bb &= !rs_to_bb(shift);

        }

        moveset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use pretty_assertions::{assert_eq, assert_ne};

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
        // starting board except with a rook at e5 (4, 4) - FEN row 3 = rank 5 = y=4
        let mut game = GameState::try_from_fen("rnbqkbnr/pppppppp/8/4R3/8/8/PPPPPPPP/RNBQKBN1").unwrap();
        game.init_magics();

        let mut expected = vec![
            // Rook at e5 (4, 4): can move along rank 5 and file e
            Move::new((4, 4), (4, 6)), // e7 (blocked by pawn)
            Move::new((4, 4), (4, 5)), // e6
            Move::new((4, 4), (4, 3)), // e4
            Move::new((4, 4), (4, 2)), // e3
            Move::new((4, 4), (0, 4)), // a5
            Move::new((4, 4), (1, 4)), // b5
            Move::new((4, 4), (2, 4)), // c5
            Move::new((4, 4), (3, 4)), // d5
            Move::new((4, 4), (5, 4)), // f5
            Move::new((4, 4), (6, 4)), // g5
            Move::new((4, 4), (7, 4)), // h5
        ];
        expected.sort();

        let mut moves: Vec<Move> = game.rook_moves().to_vec();
        moves.sort();

        assert_eq!(expected, moves);

        // more complex position: "2r2r2/pk4pp/1p6/P1p1B3/8/2R2n2/2P2P1P/1R3K2"
        // White rooks at b1 (1, 0) and c3 (2, 2)
        let mut game = GameState::try_from_fen("2r2r2/pk4pp/1p6/P1p1B3/8/2R2n2/2P2P1P/1R3K2").unwrap();
        game.init_magics();

        let mut expected = vec![
            // rook at b1 (1, 0)
            Move::new((1, 0), (0, 0)), // a1
            Move::new((1, 0), (2, 0)), // c1
            Move::new((1, 0), (3, 0)), // d1
            Move::new((1, 0), (4, 0)), // e1
            Move::new((1, 0), (1, 1)), // b2
            Move::new((1, 0), (1, 2)), // b3
            Move::new((1, 0), (1, 3)), // b4
            Move::new((1, 0), (1, 4)), // b5
            Move::new((1, 0), (1, 5)), // b6 (capture)

            // rook at c3 (2, 2)
            Move::new((2, 2), (0, 2)), // a3
            Move::new((2, 2), (1, 2)), // b3
            Move::new((2, 2), (3, 2)), // d3
            Move::new((2, 2), (4, 2)), // e3
            Move::new((2, 2), (5, 2)), // f3 (capture knight)
            Move::new((2, 2), (2, 3)), // c4
            Move::new((2, 2), (2, 4)), // c5 (capture)
        ];
        expected.sort();

        let mut moves: Vec<Move> = game.rook_moves().to_vec();
        moves.sort();

        assert_eq!(moves, expected);

        // rook at a1 (0, 0) with pawn at b2 (1, 1)
        let mut game = GameState::try_from_fen("8/8/8/8/8/8/1P6/R7").unwrap();
        game.init_magics();

        let mut expected = vec![
            Move::new((0, 0), (1, 0)), // b1
            Move::new((0, 0), (2, 0)), // c1
            Move::new((0, 0), (3, 0)), // d1
            Move::new((0, 0), (4, 0)), // e1
            Move::new((0, 0), (5, 0)), // f1
            Move::new((0, 0), (6, 0)), // g1
            Move::new((0, 0), (7, 0)), // h1
            Move::new((0, 0), (0, 1)), // a2
            Move::new((0, 0), (0, 2)), // a3
            Move::new((0, 0), (0, 3)), // a4
            Move::new((0, 0), (0, 4)), // a5
            Move::new((0, 0), (0, 5)), // a6
            Move::new((0, 0), (0, 6)), // a7
            Move::new((0, 0), (0, 7)), // a8
        ];
        expected.sort();

        let mut moves: Vec<Move> = game.rook_moves().to_vec();
        moves.sort();

        assert_eq!(moves, expected);
    }

    #[test]
    pub fn bishop_moves() {
        // Position with bishops at d3 (3, 2) and c1 (2, 0)
        // FEN: "r2qkb1r/ppp2ppp/2n2n2/4p3/3P4/2PB1R2/PP4PP/RNBQ2K1"
        let mut game = GameState::try_from_fen("r2qkb1r/ppp2ppp/2n2n2/4p3/3P4/2PB1R2/PP4PP/RNBQ2K1").unwrap();
        game.init_magics();

        let mut expected = vec![
            // bishop at c1 (2, 0)
            Move::new((2, 0), (3, 1)), // d2
            Move::new((2, 0), (4, 2)), // e3
            Move::new((2, 0), (5, 3)), // f4
            Move::new((2, 0), (6, 4)), // g5
            Move::new((2, 0), (7, 5)), // h6

            // bishop at d3 (3, 2)
            Move::new((3, 2), (2, 1)), // c2
            Move::new((3, 2), (2, 3)), // c4
            Move::new((3, 2), (1, 4)), // b5
            Move::new((3, 2), (0, 5)), // a6
            Move::new((3, 2), (4, 3)), // e4
            Move::new((3, 2), (5, 4)), // f5
            Move::new((3, 2), (6, 5)), // g6
            Move::new((3, 2), (7, 6)), // h7
            Move::new((3, 2), (4, 1)), // e2
            Move::new((3, 2), (5, 0)), // f1
        ];
        expected.sort();

        let mut moves: Vec<Move> = game.bishop_moves().to_vec();
        moves.sort();

        assert_eq!(moves, expected);
    }

    #[test]
    pub fn knight_moves() {
        // position with black knights at f6 (5, 5) and b4 (1, 3)
        let mut game = GameState::try_from_fen("r1bqr1k1/1p3pp1/p4n1p/3p4/1n1P4/2N4P/PPBQNPP1/R3R1K1").unwrap();
        game.init_magics();
        game.turn = Color::Black;

        let mut expected = vec![
            // knight at f6 (5, 5)
            Move::new((5, 5), (7, 6)),
            Move::new((5, 5), (7, 4)),
            Move::new((5, 5), (3, 6)),
            Move::new((5, 5), (6, 3)),
            Move::new((5, 5), (4, 3)),

            // knight at b4 (1, 3)
            Move::new((1, 3), (3, 2)),
            Move::new((1, 3), (2, 5)),
            Move::new((1, 3), (2, 1)),
            Move::new((1, 3), (0, 1)),
        ];
        expected.sort();

        let mut moves: Vec<Move> = game.knight_moves().to_vec();
        moves.sort();

        assert_eq!(moves, expected);
    }
}
