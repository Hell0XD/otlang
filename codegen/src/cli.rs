use std::env::args;

pub struct CLI {
    pub root: String,
    pub debug: bool,
}

impl CLI {
    pub fn new() -> CLI {
        let args = args().skip(1);

        let mut cli = CLI {
            root: String::from("./"),
            debug: false,
        };

        for arg in args {
            if arg == "--debug" {
                cli.debug = true;
            } else {
                cli.root = arg;
            }
        }

        return cli;
    }
}
