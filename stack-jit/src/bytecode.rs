#[derive(Debug, Clone, Copy)]
pub enum Bytecode {
    LocalGet(u8),
    LocalSet(u8),

    ConstI32(i32),

    Add,
    Sub,

    Call(u32),
    CallRet(u32),

    CallExtern(u8),

    If,
    Else,
    EndIf,

    IfExp,

    IfExpEq,

    Ret,

    Remove,
}
