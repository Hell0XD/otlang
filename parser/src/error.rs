use crate::{map_err, Parser};

pub fn expected_exp_or_val<'a, A>(p: impl Parser<'a, A>) -> impl Parser<'a, A> {
    map_err(p, |_| "Expected expression or a value")
}
