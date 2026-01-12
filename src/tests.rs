#[cfg(test)]
mod tests {

    use crate::chessboard;
    use crate::game::*;
    use crate::magic::*;
    use crate::movegen::*;
    use rand::{rngs::StdRng, Rng, SeedableRng};

    #[test]
    pub fn diagonals_simple() {
        let span = gen_diagonal_ray(3, 0);
        let expected: u64 = chessboard!(
                0b_00000000
                0b_00101000
                0b_01000100
                0b_10000010
                0b_00000001
                0b_00000000
                0b_00000000
                0b_00000000
        );

        assert_eq!(span, expected);

        let span: u64 = gen_diagonal_ray(1, 4);
        let expected: u64 = chessboard!(
            0b_00000100
            0b_00001000
            0b_00010000
            0b_10100000
            0b_00000000
            0b_10100000
            0b_00010000
            0b_00001000
        );

        assert_eq!(span, expected);
    }

    #[test]
    pub fn straight_simple() {
        let span: u64 = gen_straight_ray(6, 3);
        let expected = chessboard!(
            0b_00000010
            0b_00000010
            0b_00000010
            0b_11111101
            0b_00000010
            0b_00000010
            0b_00000010
            0b_00000010
        );

        assert_eq!(span, expected);

        let span: u64 = gen_straight_ray(2, 1);
        let expected = chessboard!(
            0b_00100000
            0b_11011111
            0b_00100000
            0b_00100000
            0b_00100000
            0b_00100000
            0b_00100000
            0b_00100000
        );

        assert_eq!(span, expected);
    }

    #[test]
    pub fn diagonal_blockers_simple() {
        let blockers = chessboard!(
            0b_00000000
            0b_00000000
            0b_00010000
            0b_00000010
            0b_00000000
            0b_00000010
            0b_00010000
            0b_00000000
        );
        let span: u64 = gen_blocked_diagonal(5, 4, blockers);
        let expected: u64 = chessboard!(
            0b_00000000
            0b_00000000
            0b_00010000
            0b_00001010
            0b_00000000
            0b_00001010
            0b_00010000
            0b_00000000
        );

        assert_eq!(span, expected);
    }

    #[test]
    pub fn diagonal_blockers_complex() {
        let blockers = chessboard!(
            0b_10001000
            0b_00001100
            0b_00011000
            0b_00000000
            0b_00010000
            0b_01000010
            0b_10010010
            0b_00000000
        );
        let span: u64 = gen_blocked_diagonal(3, 3, blockers);
        let expected: u64 = chessboard!(
            0b_10000000
            0b_01000000
            0b_00101000
            0b_00000000
            0b_00101000
            0b_01000100
            0b_00000010
            0b_00000000
        );

        assert_eq!(span, expected);

        let blockers = chessboard!(
            0b_00000010
            0b_00000000
            0b_01000010
            0b_00000000
            0b_01000000
            0b_00010001
            0b_00000001
            0b_00010001
        );
        let span: u64 = gen_blocked_diagonal(0, 0, blockers);
        let expected: u64 = chessboard!(
            0b_00000000
            0b_01000000
            0b_00100000
            0b_00010000
            0b_00001000
            0b_00000100
            0b_00000010
            0b_00000001
        );

        assert_eq!(span, expected);
    }

    #[test]
    pub fn straight_blockers_simple() {
        let blockers = chessboard!(
            0b_00000000
            0b_00000000
            0b_00010000
            0b_00000000
            0b_01000000
            0b_00010000
            0b_00000000
            0b_00000000
        );
        let span: u64 = gen_blocked_straight(3, 4, blockers);
        let expected: u64 = chessboard!(
            0b_00000000
            0b_00000000
            0b_00010000
            0b_00010000
            0b_01101111
            0b_00010000
            0b_00000000
            0b_00000000
        );

        assert_eq!(span, expected);

        let blockers = chessboard!(
            0b_10000001
            0b_00010000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00010000
            0b_00000010
            0b_01010000
        );
        let span: u64 = gen_blocked_straight(6, 7, blockers);
        let expected = chessboard!(
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000010
            0b_00011101
        );

        assert_eq!(span, expected);

        // weird edge case that wasn't working for some reason
        let blockers: u64 = chessboard!(
            0b_00000000
            0b_00000000
            0b_00100000
            0b_00100000
            0b_00000000
            0b_00100000
            0b_01010110
            0b_00000000
        );

        let span: u64 = gen_blocked_straight(2, 6, blockers);
        let expected: u64 = chessboard!(
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00000000
            0b_00100000
            0b_01010000
            0b_00100000
        );

        assert_eq!(span, expected);
    }

    #[test]
    pub fn knight() {
        let span: u64 = gen_knight(6, 4);
        let expected: u64 = chessboard!(
            0b_00000000
            0b_00000000
            0b_00000101
            0b_00001000
            0b_00000000
            0b_00001000
            0b_00000101
            0b_00000000
        );

        assert_eq!(span, expected);
    }

    #[test]
    pub fn magics_gen() {
        let mut rng = StdRng::seed_from_u64(0);

        for _ in 0..100 {
            let rand_board: u64 = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>();
            let straight = rng.random::<bool>();

            let x: u8 = rng.random_range(0..=7);
            let y: u8 = rng.random_range(0..=7);

            let magic_table = MagicTable::gen_table(x, y, straight);

            let clipped_ray = if straight {
                let (vertical, horizontal) = gen_straight_rays(x, y);
                (vertical & !ROW_TOP & !ROW_BOTTOM) | (horizontal & !COLUMN_LEFT & !COLUMN_RIGHT)
            } else {
                gen_diagonal_ray(x, y) & !COLUMN_LEFT & !COLUMN_RIGHT & !ROW_TOP & !ROW_BOTTOM
            };


            let blocker_board = rand_board & clipped_ray;

            let expected = if straight {
                gen_blocked_straight(x, y, blocker_board)
            } else {
                gen_blocked_diagonal(x, y, blocker_board)
            };

            // if magic_table.get_ray(blocker_board) != Some(expected) {
            //     let table_ray = magic_table.get_ray(blocker_board).unwrap_or(0);
            //     print_bitboard(blocker_board);
            //     print_bitboard(clipped_ray);
            //     print_bitboard(expected);
            //     print_bitboard(table_ray);
            // }
            assert_eq!(magic_table.get_ray(blocker_board), Some(expected));
        }
    }

    #[test]
    pub fn test_rook_moves();
}

// 0 0 0 0 0 0 0 0
// 0 0 0 0 0 0 0 0
// 0 0 0 0 0 0 0 0
// 0 0 0 0 0 0 0 0
// 0 0 0 0 0 0 0 0
// 0 0 0 0 0 0 0 0
// 0 0 0 0 0 0 0 0
// 0 0 0 0 0 0 0 0

// 0b_00000000
// 0b_00000000
// 0b_00000000
// 0b_00000000
// 0b_00000000
// 0b_00000000
// 0b_00000000
// 0b_00000000
