use std::fmt::Debug;

use crate::{bytecode::Bytecode, shape::ShapeType};

pub struct Function {
    pub args: Vec<ShapeType>,
    pub locals: Vec<ShapeType>,
    pub ret: ShapeType,
    pub code: Vec<Bytecode>,
}

impl Function {
    pub fn new(
        args: Vec<ShapeType>,
        locals: Vec<ShapeType>,
        ret: ShapeType,
        code: Vec<Bytecode>,
    ) -> Function {
        Function {
            args,
            locals,
            ret,
            code,
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}[{}] {} {} {}\n{}{}",
            "function".cyan().reset(),
            "(".bold().reset(),
            self.args
                .iter()
                .fold(String::new(), |s, arg| s + &format!(
                    "{}, ",
                    format!("{}", arg).yellow().bold().reset(),
                ))
                .trim_end_matches(", "),
            ")".bold().reset(),
            self.locals
                .iter()
                .fold(String::new(), |s, arg| s + &format!(
                    "{}, ",
                    format!("{}", arg).blue().reset()
                ))
                .trim_end_matches(", "),
            "->".cyan().reset(),
            format!("{}", self.ret).yellow().bold().reset(),
            "{".cyan().reset(),
            format_bytecode(&self.code),
            "}".cyan().reset()
        )
    }
}

fn format_bytecode(bc: &[Bytecode]) -> String {
    let mut result = String::new();
    let mut tabc = 1;

    for b in bc {
        result = (0..tabc).fold(result, |result, _| result + "    ");
        match b {
            Bytecode::If | Bytecode::IfExp => {
                tabc += 1;
                result.push_str(&format!("{:?} {{\n", b));
            }
            Bytecode::Else => {
                result.pop();
                result.pop();
                result.pop();
                result.pop();
                result.push_str(&format!("}} {:?} {{\n", b));
            }
            Bytecode::EndIf => {
                result.pop();
                result.pop();
                result.pop();
                result.pop();
                tabc -= 1;
                result.push('}');
                result.push('\n');
            }
            _ => result.push_str(&format!("{:?}\n", b)),
        }
    }
    return result;
}

trait ANSIColors {
    fn reset(&self) -> String;

    fn cyan(&self) -> String;
    fn blue(&self) -> String;
    fn yellow(&self) -> String;

    fn bold(&self) -> String;
}

impl<T: std::fmt::Display> ANSIColors for T {
    fn reset(&self) -> String {
        format!("{}\x1b[0m", self)
    }

    fn cyan(&self) -> String {
        format!("\x1b[36m{}", self)
    }

    fn blue(&self) -> String {
        format!("\x1b[34m{}", self)
    }

    fn yellow(&self) -> String {
        format!("\x1b[33m{}", self)
    }

    fn bold(&self) -> String {
        format!("\x1b[1m{}", self)
    }
}
