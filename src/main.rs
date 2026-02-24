use arrayvec::ArrayVec;
use knightmare::{chessboard, game::GameState, magic::*, movegen::*};
use rand::Rng;

fn main() {
    println!("Horsie v{}", env!("CARGO_PKG_VERSION"));
    println!("By Rift");
    if let Ok(art) = std::fs::read_to_string("assets/art.txt") {
        println!("{art}");
    }

    let a: u64 = 0b1;
    let b = a.overflowing_shl(64);

    println!("{:?} {:?}", a, b);

    // for _ in 0..100_000_000 {
    //     let x = rng.gen_range(0..8);
    //     let y = rng.gen_range(0..8);

    //     let mut blockers: u64 = u64::MAX;
    //     for _ in 0..3 {
    //         let r: u64 = rng.random();
    //         blockers = blockers & r;
    //     }
    //     let span = gen_blocked_diagonal(x, y, blockers);
    //     let span = gen_blocked_straight(x, y, blockers);
    // }

    // this looks ugly as fuck
    // let mut game = GameState::new();
    // game.init_magics();


    // generate_magics();

    // let rook_moves = game.rook_moves();
    // println!("{:?}", rook_moves);


    // boo! hello thereeeeee.......
}

pub fn generate_magics() -> ([[MagicTable; 8]; 8], [[MagicTable; 8]; 8]) {
    let mut straight_magics: [[MagicTable; 8]; 8] = Default::default();
    let mut diagonal_magics: [[MagicTable; 8]; 8] = Default::default();


    for x in 0..8 {
        for y in 0..8 {
            let table = MagicTable::gen_table(x, y, true);
            straight_magics[x][y] = table;
            let table = MagicTable::gen_table(x, y, false);
            diagonal_magics[x][y] = table;
        }
    }

    (straight_magics, diagonal_magics)
}
