

#[cfg(test)]
mod tests {

    use crate::game::*;
    use crate::magic::*;
    use crate::movegen::*;
    use crate::chessboard;
    use rand::{SeedableRng, Rng, rngs::StdRng};

    #[test]
    pub fn test_diagonals() {
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
    pub fn test_straight() {

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
    pub fn test_diagonal_blockers() {

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
    pub fn test_diagonal_blockers_complex() {

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
    pub fn test_straight_blockers() {

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
    }


    #[test]
    pub fn test_knight() {

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
    pub fn test_magics_gen() {

        let ray = gen_diagonal_ray(2, 4);
        let (table, magic) = gen_magic_table(2, 4, false);

        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..1_000 {
            let rand_board: u64 = rng.random::<u64>();
            let mut blocker_board = rand_board & ray;

            blocker_board &= !column_left;
            blocker_board &= !(column_left >> 7);
            blocker_board &= !row_top;
            blocker_board &= !(row_top >> 56);

            let table_sz = 10;

            let map_index = (blocker_board.wrapping_mul(magic) >> (64-table_sz)) as usize;
            assert!(table[map_index] != 0);
        }
    }
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
