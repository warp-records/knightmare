fn main() {
    println!("Horsie v{}", env!("CARGO_PKG_VERSION"));
    println!("By Rift");
    if let Ok(art) = std::fs::read_to_string("assets/art.txt") {
        println!("{art}");
    }

    // boo! hello thereeeeee.......
}
