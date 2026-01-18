use arrayvec::ArrayVec;
use horsie::{chessboard, game::GameState, magic::*, movegen::*};
use rand::Rng;

fn main() {
    println!("Horsie v{}", env!("CARGO_PKG_VERSION"));
    println!("By Rift");
    if let Ok(art) = std::fs::read_to_string("assets/art.txt") {
        println!("{art}");
    }

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
    let mut game = GameState::new();
    game.init_magics();

    // boo! hello thereeeeee.......
}

pub fn generate_magics() -> (Vec<[MagicTable; 8]>, Vec<[MagicTable; 8]>) {

    let mut straight_magics: Vec<[MagicTable; 8]> = (0..8)
        .map(|_| std::array::from_fn(|_| MagicTable::default()))
        .collect();

    let mut diagonal_magics: Vec<[MagicTable; 8]> = (0..8)
        .map(|_| std::array::from_fn(|_| MagicTable::default()))
        .collect();

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
