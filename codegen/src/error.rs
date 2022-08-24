pub struct CompilerError<'a> {
    msg: String,
    file: &'a str,
    line: usize,
    char: usize,
}

impl<'a> CompilerError<'a> {
    pub fn new(msg: impl Into<String>, file: &str, line: usize, char: usize) -> CompilerError {
        CompilerError {
            msg: msg.into(),
            file,
            line,
            char,
        }
    }
}

impl<'a> error::Error for CompilerError<'a> {
    fn get(&self) -> (String, &str, usize, usize) {
        (self.msg.clone(), self.file, self.line, self.char)
    }
}
