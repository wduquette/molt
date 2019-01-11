use gcl::Interp;

fn main() {
    let mut interp = Interp::new();

    gcl::shell(&mut interp, "% ");
}
