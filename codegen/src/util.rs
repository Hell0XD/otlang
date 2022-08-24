#[derive(Clone, Copy, PartialEq)]
enum Escaped {
    True,
    False,
}

pub fn escape_string(s: &str) -> String {
    s.chars()
        .scan(Escaped::False, |ctx, char| {
            if char == '\\' && *ctx == Escaped::False {
                *ctx = Escaped::True;
                Some(0 as char)
            } else if *ctx == Escaped::True {
                *ctx = Escaped::False;
                Some(match char {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '0' => '\0',
                    '\\' => '\\',
                    _ => return None,
                })
            } else {
                Some(char)
            }
        })
        .filter(|&char| char != 0 as char)
        .collect::<String>()
}
