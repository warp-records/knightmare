use arrayvec::*;



#[derive(PartialEq)]
enum Color { Black, White }

enum PieceType {
   King(Color),
   Queen(Color),
   Rook(Color),
   Bishop(Color),
   Horses(Color),
   Pawn(Color, bool)
}

pub struct GameState {
    black: ColorSet,
    white: ColorSet,
    turn: Color,
    win: Option<Color>,
}

/// struct representing board state of a given color
pub struct ColorSet {
    king: u64,
    queens: u64,
    rooks: u64,
    bishops: u64,
    horses: u64,
    pawns: u64,
    // bitboard representing any pawn which previously moved two squares
    en_passe: u64,
    moved_rooks: u64,
    king_moved: bool,
    /// positions of all piece
    all: u64,
    /// mask of threat lines of all pieces
    threats: u64,
}



impl GameState {
    // (moves list, threat lines)
    pub fn king_moves(&self) -> (ArrayVec<u64, 8>, u64) {
        let mut moves = ArrayVec::new();

        let king_bb = if self.turn == Color::Black { self.black.king } else { self.white.king };
        let opponent_threats = if self.turn == Color::Black { self.white.threats } else { self.black.threats };

        let mut move_mask: u64 = 0xE0A0E00000000000;
        let move_mask_pos = 9;
        let self_threat: u64 = if king_bb.leading_zeros() > move_mask_pos {
            move_mask >> king_bb.leading_zeros() - move_mask_pos
        } else {
            move_mask >>  move_mask_pos - king_bb.leading_zeros()
        };


        // up left
        moves.push(king_bb << (8 + 1));
        // up
        moves.push(king_bb << 8);
        // up right
        moves.push(king_bb << (8 - 1));
        // etc
        moves.push(king_bb << 1);
        moves.push(king_bb >> 1);
        moves.push(king_bb >> (8 - 1));
        moves.push(king_bb >> 8);
        moves.push(king_bb >> (8 + 1));

        moves.retain(|bb| (*bb&opponent_threats == 0));
        (moves, self_threat)
    }

    // Return order is from top to bottom, left to right formatted as (original_pos_bitboard, new_pos_bitboard)
    // implement pawn promotion later
    // pub fn gen_pawn_moves() -> ArrayVec<(u64, 64), 24> {


    //     ArrayVec::new()
    // }
}
