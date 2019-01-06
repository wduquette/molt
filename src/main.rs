fn main() {
    let input = String::from("  some words  \nanother");
    let chars = &mut input.chars();

    while let Some(cmd) = gcl::parse_command(chars) {
        println!("Got: {:?}", cmd);
    }
}
