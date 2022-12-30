extern crate cc;

fn main() {
    cc::Build::new().file("examples/read.c").compile("read");
}
