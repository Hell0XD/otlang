use codegen::cli::CLI;
use codegen::loader;

use std::io::Write;

pub fn main() {
    let cli = CLI::new();
    let serialized = loader::load_files(&cli.root, cli.debug);

    let mut file = std::fs::File::create("./.out").unwrap();
    file.write_all(&serialized).unwrap();
}
