use crate::{
    asm::{
        self,
        arg::{Arg, Register},
        buffer::Buffer,
    },
    bytecode::Bytecode,
    compiler::{compile, JIT},
    function::Function,
    shape::ShapeType,
};

#[test]
fn test_asm() {
    let mut buff = Buffer::new();

    asm::push(&mut buff, Arg::Memory(Register::RSP, 5));

    println!("{:02X?}", buff.get());
}

#[test]
fn test_stack_args() {
    let mut jit = JIT::new();

    let mut code = vec![];

    code.push(Bytecode::LocalGet(0));
    code.push(Bytecode::LocalGet(4));
    code.push(Bytecode::Add);
    code.push(Bytecode::Ret);

    let function = Function::new(vec![ShapeType::Int; 5], vec![], ShapeType::Int, code);

    compile(&mut jit, 0, function).unwrap();
    jit.print_machine_code();

    let res = jit.call(0, &[6, 0, 0, 0, 4]);
    println!("RES = {:?}", res);
}

#[test]
fn test_if_else() {
    let mut jit = JIT::new();

    let mut code = vec![];

    code.push(Bytecode::LocalGet(1));
    code.push(Bytecode::LocalGet(2));
    code.push(Bytecode::LocalGet(0));

    code.push(Bytecode::IfExp);
    code.push(Bytecode::Add);
    code.push(Bytecode::Else);
    code.push(Bytecode::Add);
    code.push(Bytecode::EndIf);

    code.push(Bytecode::Ret);

    let function = Function::new(vec![ShapeType::Int; 3], vec![], ShapeType::Int, code);

    compile(&mut jit, 0, function).unwrap();
    jit.print_machine_code();

    let res = jit.call(0, &[0, 5, 4]).unwrap();
    println!("RES={}", res);
}

#[test]
fn test_extern_call() {
    extern "sysv64" fn test_api() -> i32 {
        println!("HELLO");
        69
    }

    let mut jit = JIT::new();
    jit.add_extern_function(test_api as u64, vec![], ShapeType::Int);

    let mut code = vec![];

    code.push(Bytecode::CallExtern(0));
    code.push(Bytecode::Call(0));
    code.push(Bytecode::Ret);

    let function = Function::new(vec![ShapeType::Int; 0], vec![], ShapeType::Int, code);

    compile(&mut jit, 0, function).unwrap();
    jit.print_machine_code();

    let res = jit.call(0, &[]).unwrap();
    println!("RES={}", res);
}

#[test]
fn test_call_recursive() {
    extern "sysv64" fn test_api(i: i32) -> i32 {
        println!("{}", i);
        i + 1
    }

    let mut jit = JIT::new();
    jit.add_extern_function(test_api as u64, vec![ShapeType::Int], ShapeType::Int);

    let mut code = vec![];

    code.push(Bytecode::LocalGet(0));
    code.push(Bytecode::CallExtern(0));
    code.push(Bytecode::CallRet(0));

    let function = Function::new(vec![ShapeType::Int; 1], vec![], ShapeType::Int, code);

    compile(&mut jit, 0, function).unwrap();
    jit.print_machine_code();

    let res = jit.call(0, &[0]).unwrap();
    println!("RES={}", res);
}

#[test]
fn t_call() {
    let mut jit = JIT::new();

    compile(
        &mut jit,
        1,
        Function::new(
            vec![ShapeType::Int; 1],
            vec![],
            ShapeType::Int,
            vec![
                Bytecode::LocalGet(0),
                Bytecode::LocalGet(0),
                Bytecode::Add,
                Bytecode::Ret,
            ],
        ),
    )
    .unwrap();

    let mut code = vec![];

    code.push(Bytecode::LocalGet(0));
    code.push(Bytecode::Call(1));
    code.push(Bytecode::Ret);

    let function = Function::new(vec![ShapeType::Int; 1], vec![], ShapeType::Int, code);

    compile(&mut jit, 0, function).unwrap();
    jit.print_machine_code();

    let f = jit.cast_1(0).unwrap();
    println!("RES={}", f(4));
}

#[test]
fn test_fib() {
    let mut jit = JIT::new();

    let mut code = vec![];

    code.push(Bytecode::LocalGet(0));
    code.push(Bytecode::ConstI32(0));
    code.push(Bytecode::IfExpEq);

    code.push(Bytecode::LocalGet(0));
    code.push(Bytecode::Else);

    code.push(Bytecode::LocalGet(0));
    code.push(Bytecode::ConstI32(1));
    code.push(Bytecode::IfExpEq);

    code.push(Bytecode::LocalGet(0));
    code.push(Bytecode::Else);

    code.push(Bytecode::EndIf);
    code.push(Bytecode::EndIf);

    let function = Function::new(vec![ShapeType::Int], vec![], ShapeType::Int, code);

    compile(&mut jit, 0, function).unwrap();
    jit.print_machine_code();

    let f = jit.cast_1(0).unwrap();
    println!("{}", f(9));
}

#[test]
fn test_2_imm() {
    let mut jit = JIT::new();

    let mut code = vec![];

    code.push(Bytecode::ConstI32(4));
    code.push(Bytecode::ConstI32(6));
    code.push(Bytecode::Add);
    code.push(Bytecode::Ret);

    let function = Function::new(vec![], vec![], ShapeType::Int, code);

    compile(&mut jit, 0, function).unwrap();
    jit.print_machine_code();

    let f = jit.cast_0(0).unwrap();
    println!("{}", f() as i8);
}

#[test]
fn t() {
    let mut jit = JIT::new();

    let mut code = vec![];

    code.push(Bytecode::LocalGet(0));
    code.push(Bytecode::ConstI32(6));
    code.push(Bytecode::Add);
    code.push(Bytecode::Ret);

    let function = Function::new(vec![ShapeType::Int; 1], vec![], ShapeType::Int, code);

    compile(&mut jit, 0, function).unwrap();
    jit.print_machine_code();

    let f = jit.cast_1(0).unwrap();
    println!("{}", f(4));
}
