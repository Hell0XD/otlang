use error::Error;

#[derive(Debug, PartialEq)]
pub struct ParserError<'a> {
    recoverable: bool,
    msg: String,

    line: usize,
    char: usize,

    file: &'a str,
}

impl<'a> ParserError<'a> {
    pub fn new(msg: impl Into<String>, file: &'a str, line: usize, char: usize) -> ParserError {
        ParserError {
            recoverable: true,
            msg: msg.into(),
            file,
            line,
            char,
        }
    }
}

impl<'a> Error for ParserError<'a> {
    fn get(&self) -> (String, &str, usize, usize) {
        (self.msg.clone(), self.file, self.line, self.char)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FileCxt<'a> {
    pub s: &'a str,
    pub line: usize,
    pub char: usize,
    pub file: &'a str,

    pub module: &'a str,
}

impl<'a> FileCxt<'a> {
    pub fn update(mut self, new_s: &'a str, new_line: usize, new_char: usize) -> Self {
        self.s = new_s;
        self.line += new_line;
        if new_line == 0 {
            self.char += new_char;
        } else {
            self.char = new_char;
        }
        self
    }

    pub fn new_test(s: &'a str, file: &'a str, line: usize, char: usize) -> Self {
        FileCxt {
            s,
            file,
            line,
            char,
            module: "",
        }
    }

    pub fn new(s: &'a str, file: &'a str) -> Self {
        FileCxt {
            s,
            file,
            line: 1,
            char: 0,
            module: "",
        }
    }
}

pub type ParserResult<'a, R> = Result<(FileCxt<'a>, R), ParserError<'a>>;

pub trait Parser<'a, R> {
    fn parse(&self, input: FileCxt<'a>) -> ParserResult<'a, R>;
}

impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(FileCxt<'a>) -> ParserResult<Output>,
{
    fn parse(&self, input: FileCxt<'a>) -> ParserResult<'a, Output> {
        self(input)
    }
}

// Modifiers

pub fn map<'a, A, B>(parser: impl Parser<'a, A>, f: impl Fn(A) -> B) -> impl Parser<'a, B> {
    move |input| parser.parse(input).map(|(input, a)| (input, f(a)))
}

pub fn non_recoverable<'a, A>(parser: impl Parser<'a, A>) -> impl Parser<'a, A> {
    move |input| {
        parser.parse(input).map_err(
            |ParserError {
                 recoverable: _,
                 msg,
                 line,
                 char,
                 file,
             }: ParserError| ParserError {
                recoverable: false,
                msg,
                line,
                char,
                file,
            },
        )
    }
}

pub fn map_err<'a, A, S>(parser: impl Parser<'a, A>, f: impl Fn(&str) -> S) -> impl Parser<'a, A>
where
    S: Into<String>,
{
    move |input| {
        parser.parse(input).map_err(
            |ParserError {
                 recoverable,
                 msg,
                 line,
                 char,
                 file,
             }: ParserError| ParserError {
                recoverable,
                msg: f(&msg).into(),
                line,
                char,
                file,
            },
        )
    }
}

pub fn and_then<'a, A, B>(
    parser: impl Parser<'a, A>,
    f: impl Fn(A) -> Result<B, ParserError<'a>>,
) -> impl Parser<'a, B> {
    move |input| {
        parser
            .parse(input)
            .and_then(|(input, val)| Ok((input, f(val)?)))
    }
}

// Combinators

pub fn pair<'a, A, B>(
    parser1: impl Parser<'a, A>,
    parser2: impl Parser<'a, B>,
) -> impl Parser<'a, (A, B)> {
    move |input| {
        parser1.parse(input).and_then(|(input, val1)| {
            parser2
                .parse(input)
                .map(move |(input, val2)| (input, (val1, val2)))
        })
    }
}

pub fn left<'a, A, B>(
    parser1: impl Parser<'a, A>,
    parser2: impl Parser<'a, B>,
) -> impl Parser<'a, A> {
    map(pair(parser1, parser2), |(left, _)| left)
}

pub fn right<'a, A, B>(
    parser1: impl Parser<'a, A>,
    parser2: impl Parser<'a, B>,
) -> impl Parser<'a, B> {
    map(pair(parser1, parser2), |(_, right)| right)
}

// Optional Combinators

pub fn or<'a, A>(parser1: impl Parser<'a, A>, parser2: impl Parser<'a, A>) -> impl Parser<'a, A> {
    move |input| {
        parser1.parse(input).or_else(|err| {
            if err.recoverable {
                parser2.parse(input)
            } else {
                Err(err)
            }
        })
    }
}

pub fn maybe<'a, A>(parser: impl Parser<'a, A>, default: impl Fn() -> A) -> impl Parser<'a, A> {
    move |input| parser.parse(input).or_else(|_| Ok((input, default())))
}

// Repeaters

pub fn zero_or_more<'a, A>(parser: impl Parser<'a, A>) -> impl Parser<'a, Vec<A>> {
    move |mut input| {
        let mut vec = Vec::new();

        while let Ok((_input, val)) = parser.parse(input) {
            input = _input;
            vec.push(val);
        }

        return Ok((input, vec));
    }
}

pub fn one_or_more<'a, A>(parser: impl Parser<'a, A>) -> impl Parser<'a, Vec<A>> {
    move |input| {
        let (input, val) = parser.parse(input)?;

        let (input, mut vec) = zero_or_more(|input| parser.parse(input)).parse(input)?;
        vec.insert(0, val);

        return Ok((input, vec));
    }
}
