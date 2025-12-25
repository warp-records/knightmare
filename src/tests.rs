

#[cfg(test)]
mod tests {

    use crate::game::*;
    use crate::magic::*;
    use crate::chessboard;

    #[test]
    pub fn test_diagonals() {
        let span = gen_diagonal_ray(3, 0);
        let expected: u64 = chessboard!(
                0b_00010000
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
            0b_01000000
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
            0b_11111111
            0b_00000010
            0b_00000010
            0b_00000010
            0b_00000010
        );

        assert_eq!(span, expected);

        let span: u64 = gen_straight_ray(2, 1);
        let expected = chessboard!(
            0b_00100000
            0b_11111111
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
        let span: u64 = gen_clipped_diagonal(5, 4, blockers);
        let expected: u64 = chessboard!(
            0b_00000000
            0b_00000000
            0b_00010000
            0b_00001010
            0b_00000100
            0b_00001010
            0b_00010000
            0b_00000000
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
