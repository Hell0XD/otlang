use super::{map, or, pair, right, zero_or_more, FileCxt, Parser, ParserError};

#[derive(Debug, PartialEq)]
pub struct Identifier<'a> {
    pub val: &'a str,
    pub line: usize,
    pub start: usize,
    pub end: usize,
    pub file: &'a str,
}

/// Skips whitespaces on left
pub fn __<'a, A>(parser: impl Parser<'a, A>) -> impl Parser<'a, A> {
    right(
        map(
            zero_or_more(or(
                map(identifier(|ch| ch.is_whitespace()), |_| ()),
                comment(),
            )),
            |_| (),
        ),
        parser,
    )
}

fn comment<'a>() -> impl Parser<'a, ()> {
    map(
        pair(match_literal("#"), identifier(|ch| ch != '\n')),
        |_| (),
    )
}

pub fn match_literal<'a>(literal: &'static str) -> impl Parser<'a, ()> {
    move |ctx: FileCxt<'a>| {
        if ctx.s.starts_with(literal) {
            Ok((
                ctx.update(
                    &ctx.s[literal.len()..],
                    literal.chars().filter(|&ch| ch == '\n').count(),
                    literal[literal.rfind("\n").map(|p| p + 1).unwrap_or(0)..]
                        .chars()
                        .count(),
                ),
                (),
            ))
        } else {
            Err(ParserError::new(
                format!("Expected `{}`", literal),
                ctx.file,
                ctx.line,
                ctx.char,
            ))
        }
    }
}

pub fn identifier<'a>(valid_char: impl Fn(char) -> bool) -> impl Parser<'a, Identifier<'a>> {
    move |ctx: FileCxt<'a>| {
        if let Some(next) = ctx.s.chars().next() {
            if !valid_char(next) {
                return Err(ParserError::new(
                    format!("Unexpected character `{}`", next),
                    ctx.file,
                    ctx.line,
                    ctx.char,
                ));
            }
        } else {
            return Err(ParserError::new(
                "Unexpected end of file",
                ctx.file,
                ctx.line,
                ctx.char,
            ));
        }

        let mut index = 0;
        let mut chars = ctx.s.chars();

        while let Some(next) = chars.next() {
            if !valid_char(next) {
                break;
            }

            index += next.len_utf8();
        }

        let used = &ctx.s[0..index];

        let new_char = used[used.rfind("\n").map(|p| p + 1).unwrap_or(0)..]
            .chars()
            .count();

        let id = Identifier {
            val: &used[0..index],
            line: ctx.line,
            start: ctx.char + 1,
            end: ctx.char + 1 + new_char,
            file: ctx.file,
        };

        return Ok((
            ctx.update(
                &ctx.s[index..],
                used.chars().filter(|&ch| ch == '\n').count(),
                new_char,
            ),
            id,
        ));
    }
}
