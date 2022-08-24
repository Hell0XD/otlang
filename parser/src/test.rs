use error::Error;

use crate::{parse, FileCxt, Parser};

#[test]
fn t() {
    let path = "test.spsl";
    let content = std::fs::read_to_string(path).unwrap();
    let result = parse().parse(FileCxt::new(&content, path));

    match result {
        Ok((_, blocks)) => println!("{:?}", blocks),
        Err(e) => e.display(),
    }
}
