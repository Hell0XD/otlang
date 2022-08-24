mod base;
mod blocks;
mod error;
mod keyword;
mod macros;
mod string;
mod value;

#[cfg(test)]
mod test;

#[allow(unused)]
pub use base::*;
pub use blocks::*;
pub use keyword::*;
pub use string::*;
pub use value::*;

pub fn parse<'a>() -> impl Parser<'a, Vec<Block<'a>>> {
    |mut ctx: FileCxt<'a>| {
        let (ctx_, module) = __(right(
            match_literal("mod"),
            __(identifier(|char| char.is_ascii_alphabetic())),
        ))
        .parse(ctx)
        .unwrap();
        ctx = ctx_;
        ctx.module = module.val;

        let mut v = Vec::new();
        while !__(|ctx| Ok((ctx, ()))).parse(ctx).unwrap().0.s.is_empty() {
            let (ctx_, b) = block().parse(ctx)?;
            ctx = ctx_;
            v.push(b);
        }
        return Ok((ctx, v));
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::{base::Parser, FileCxt};

    #[test]
    fn parse_test() {
        let blocks = parse()
            .parse(FileCxt::new_test(
                "(lambda (x y) (+ x y))",
                "test.spsl",
                0,
                1,
            ))
            .unwrap()
            .1;

        println!("{:?}", blocks);
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
