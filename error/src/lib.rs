#[macro_export]
macro_rules! raise_vm_error {
    ($($t: tt)+) => {
        {
            println!("\x1b[31m\x1b[1mVM ERROR:\x1b[0m {}", format!($($t)+));
            std::process::exit(0);
        }
    };
}

pub trait Error {
    fn display(&self) {
        let (msg, file, line, col) = self.get();

        let content = std::fs::read_to_string(file).unwrap();

        println!(
            "\x1b[31m{}\x1b[0m on line {} and col {}; {}:{}:{}",
            msg, line, col, file, line, col
        );

        let (to_skip, offset) = if let Some(skip) = line.checked_sub(2) {
            (skip, 1)
        } else {
            (0, 0)
        };

        content
            .split("\n")
            .skip(to_skip)
            .take(5)
            .enumerate()
            .for_each(|(index, curr_line)| {
                if index == offset {
                    println!(
                        "\x1b[31m\x1b[1m{}:\t {}\x1b[0m",
                        (line + index) - 1,
                        curr_line
                            .chars()
                            .enumerate()
                            .map(|(i, ch)| {
                                if i == col - 1 {
                                    format!("\x1b[41m\x1b[37m\x1b[4m{}\x1b[0m\x1b[31m\x1b[1m", ch)
                                } else {
                                    ch.to_string()
                                }
                            })
                            .collect::<String>()
                    );
                } else {
                    println!("{}:\t {}", (line + index) - 1, curr_line);
                }
            });

        std::process::exit(1);
    }

    fn get_formated_with_file(&self, content: &str) -> String {
        let (msg, file, line, col) = self.get();

        let mut result = String::new();

        result += &format!(
            "\x1b[31m{}\x1b[0m on line {} and col {}; {}:{}:{}",
            msg, line, col, file, line, col
        );

        let (to_skip, offset) = if let Some(skip) = line.checked_sub(2) {
            (skip, 1)
        } else {
            (0, 0)
        };

        content
            .split("\n")
            .skip(to_skip)
            .take(5)
            .enumerate()
            .for_each(|(index, curr_line)| {
                if index == offset {
                    result += &format!(
                        "\x1b[31m\x1b[1m{}:\t {}\x1b[0m",
                        (line + index) - 1,
                        curr_line
                            .chars()
                            .enumerate()
                            .map(|(i, ch)| {
                                if i == col - 1 {
                                    format!("\x1b[41m\x1b[37m\x1b[4m{}\x1b[0m\x1b[31m\x1b[1m", ch)
                                } else {
                                    ch.to_string()
                                }
                            })
                            .collect::<String>()
                    );
                } else {
                    result += &format!("{}:\t {}", (line + index) - 1, curr_line);
                }
            });

        return result;
    }

    // Msg, File, Line, Char
    fn get(&self) -> (String, &str, usize, usize);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
