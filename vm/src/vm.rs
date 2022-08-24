use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use crate::deserialize::Type as T;

#[cfg(not(target_arch = "wasm32"))]
use crate::dlibraries::DynamicLibraries;

use crate::helper::IsIf;
use crate::value::Value;

#[cfg(not(target_arch = "wasm32"))]
use crate::jit::typecheck::Type;

use crate::vm_runtime_error;
use vm_bytecode::Bytecode;

#[cfg(not(target_arch = "wasm32"))]
use crate::jit::JITHandle;

const DEFAULT_STACK_SIZE: usize = 256;

pub struct VM<F: Fn(&str)> {
    constants: Vec<Value>,
    functions: Vec<(u8, u8, Vec<Bytecode>)>,

    signatures: Vec<(String, Vec<T>, T)>,
    groups: Vec<Vec<u32>>,

    #[cfg(not(target_arch = "wasm32"))]
    jit: JITHandle,
    #[cfg(not(target_arch = "wasm32"))]
    dl: DynamicLibraries,

    print_handle: F,
    should_jit: bool,
    debug_jit: bool,
}

impl<F: Fn(&str)> VM<F> {
    pub fn new(
        constants: Vec<Value>,
        functions: Vec<(u8, u8, Vec<Bytecode>)>,
        signatures: Vec<(String, Vec<T>, T)>,
        groups: Vec<Vec<u32>>,
        print_handle: F,
        should_jit: bool,
        debug_jit: bool,
    ) -> VM<F> {
        let fc = functions.len();

        VM {
            constants,
            functions,
            signatures,
            groups,
            should_jit,
            debug_jit,

            #[cfg(not(target_arch = "wasm32"))]
            jit: JITHandle::new(fc),
            #[cfg(not(target_arch = "wasm32"))]
            dl: DynamicLibraries::new(),
            print_handle,
        }
    }

    pub fn run(&mut self, start_function: u32, mut args: Vec<Value>) -> Value {
        let (_, localc, code) = self.borrow_function(start_function);
        let mut code = code;

        args.extend(vec![Value::Int(0); *localc as usize]);
        let mut locals = args;

        const V: Value = Value::Int(0);
        let mut stack = [V; DEFAULT_STACK_SIZE];
        let mut stack_pointer = 0;

        let mut pc = 0;

        while pc < code.len() {
            match code[pc] {
                Bytecode::StringLen => {
                    let s = Self::pop_stack(&mut stack, &mut stack_pointer);

                    if let Value::String(s) = s {
                        Self::push_stack(
                            &mut stack,
                            &mut stack_pointer,
                            Value::Int(s.as_bytes().len() as i32),
                        );
                    } else {
                        unreachable!();
                    }
                }
                Bytecode::Import(index) => {
                    let path = Self::pop_stack(&mut stack, &mut stack_pointer);

                    if let Value::String(path) = path {
                        let sigs = &self.groups[index as usize];

                        let symbols: Vec<String> = sigs
                            .into_iter()
                            .map(|sig| self.signatures[*sig as usize].0.clone())
                            .collect();

                        #[cfg(not(target_arch = "wasm32"))]
                        self.dl.load(path.to_string_lossy().borrow(), symbols);
                    }

                    Self::push_stack(&mut stack, &mut stack_pointer, Value::Nil);
                }
                Bytecode::CallDynamic(index) => {
                    let (name, args, ret) = &self.signatures[index as usize];
                    let argc = args.len();

                    let mut args = Vec::with_capacity(argc as usize);

                    for _ in 0..argc {
                        args.push(Self::pop_stack(&mut stack, &mut stack_pointer));
                    }

                    args.reverse();

                    #[cfg(not(target_arch = "wasm32"))]
                    let symbol = self.dl.get_symbol(name).expect("Unknown symbol");

                    use std::mem::transmute;

                    #[cfg(not(target_arch = "wasm32"))]
                    let return_value = unsafe {
                        match argc {
                            0 => transmute::<_, extern "C" fn() -> i64>(symbol)(),
                            1 => transmute::<_, extern "C" fn(i64) -> i64>(symbol)(args[0].ffi()),
                            2 => transmute::<_, extern "C" fn(i64, i64) -> i64>(symbol)(
                                args[0].ffi(),
                                args[1].ffi(),
                            ),
                            _ => unimplemented!("Can't call this symbol UnU"),
                        }
                    };
                    #[cfg(target_arch = "wasm32")]
                    let return_value: i64 = 0;

                    #[cfg(target_arch = "wasm32")]
                    let val = Value::Nil;

                    #[cfg(not(target_arch = "wasm32"))]
                    let val = match ret {
                        T::Int => Value::Int(return_value as i32),
                        T::String => {
                            let ptr = return_value as *const i8;
                            let s = unsafe { std::ffi::CStr::from_ptr(ptr) };
                            let v = Value::String(s.to_owned());

                            unsafe { libc::free(ptr as *mut libc::c_void) };
                            v
                        }
                    };

                    Self::push_stack(&mut stack, &mut stack_pointer, val);
                }

                Bytecode::LocalGet(i) => {
                    Self::push_stack(&mut stack, &mut stack_pointer, locals[i as usize].clone())
                }
                Bytecode::LocalSet(i) => {
                    locals[i as usize] = Self::pop_stack(&mut stack, &mut stack_pointer)
                }
                Bytecode::ConstantGet(i) => Self::push_stack(
                    &mut stack,
                    &mut stack_pointer,
                    self.constants[i as usize].clone(),
                ),
                Bytecode::ConstantNil => {
                    Self::push_stack(&mut stack, &mut stack_pointer, Value::Nil)
                }
                Bytecode::Add => {
                    let first = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let second = Self::pop_stack(&mut stack, &mut stack_pointer);
                    Self::push_stack(&mut stack, &mut stack_pointer, first + second);
                }
                Bytecode::Sub => {
                    let first = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let second = Self::pop_stack(&mut stack, &mut stack_pointer);
                    Self::push_stack(&mut stack, &mut stack_pointer, second - first);
                }
                Bytecode::Mul => {
                    let first = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let second = Self::pop_stack(&mut stack, &mut stack_pointer);
                    Self::push_stack(&mut stack, &mut stack_pointer, first * second);
                }
                Bytecode::Div => {
                    let first = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let second = Self::pop_stack(&mut stack, &mut stack_pointer);
                    Self::push_stack(&mut stack, &mut stack_pointer, second / first);
                }
                Bytecode::Eq => {
                    let first = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let second = Self::pop_stack(&mut stack, &mut stack_pointer);
                    Self::push_stack(
                        &mut stack,
                        &mut stack_pointer,
                        Value::Int((first == second) as i32),
                    );
                }
                Bytecode::Lt => {
                    let first = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let second = Self::pop_stack(&mut stack, &mut stack_pointer);
                    Self::push_stack(
                        &mut stack,
                        &mut stack_pointer,
                        Value::Int((second < first) as i32),
                    );
                }
                Bytecode::Gt => {
                    let first = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let second = Self::pop_stack(&mut stack, &mut stack_pointer);
                    Self::push_stack(
                        &mut stack,
                        &mut stack_pointer,
                        Value::Int((second > first) as i32),
                    );
                }
                Bytecode::Lte => {
                    let first = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let second = Self::pop_stack(&mut stack, &mut stack_pointer);
                    Self::push_stack(
                        &mut stack,
                        &mut stack_pointer,
                        Value::Int((second <= first) as i32),
                    );
                }
                Bytecode::Gte => {
                    let first = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let second = Self::pop_stack(&mut stack, &mut stack_pointer);
                    Self::push_stack(
                        &mut stack,
                        &mut stack_pointer,
                        Value::Int((second >= first) as i32),
                    );
                }
                Bytecode::If => {
                    let cond = Self::pop_stack(&mut stack, &mut stack_pointer);
                    if cond == Value::Int(0) || cond == Value::Nil {
                        let mut fc: u32 = 0;

                        loop {
                            pc += 1;
                            match code[pc] {
                                bc if bc.is_if() => fc += 1,
                                Bytecode::Else if fc == 0 => break,
                                Bytecode::Else if fc != 0 => fc -= 1,
                                _ => (),
                            }
                        }
                    }
                }
                Bytecode::Else => {
                    let mut elsec: u32 = 0;

                    loop {
                        pc += 1;
                        match code[pc] {
                            Bytecode::Else => elsec += 1,
                            Bytecode::End if elsec == 0 => break,
                            Bytecode::End if elsec != 0 => elsec -= 1,
                            _ => (),
                        }
                    }
                }
                Bytecode::End => (),
                Bytecode::Call(i) => {
                    let i_usize = i as usize;

                    #[cfg(not(target_arch = "wasm32"))]
                    let result = if self.jit.function_exist(i) {
                        println!("CALLING JIT");
                        let (argc, _, _) = &self.functions[i_usize];

                        let mut args = Vec::with_capacity(*argc as usize);
                        for _ in 0..*argc {
                            args.push(Self::pop_stack(&mut stack, &mut stack_pointer).into());
                        }

                        args.reverse();

                        let shape = self.jit.get_shape(i_usize);
                        Value::from((shape.ret, self.jit.call_function(i, &args).unwrap()))
                    } else {
                        if self.jit.is_function_hot(i_usize, 5) {
                            let (_, _, bc) = &self.functions[i_usize];

                            let shape = self.jit.get_shape(i_usize);
                            self.jit.compile(
                                i_usize,
                                shape.args.unwrap(),
                                0,
                                shape.ret,
                                bc,
                                &self.constants,
                                self.debug_jit,
                            );
                        }

                        let (argc, _, _) = &self.functions[i_usize];
                        let mut args = Vec::with_capacity(*argc as usize);

                        for _ in 0..*argc {
                            args.push(Self::pop_stack(&mut stack, &mut stack_pointer));
                        }

                        args.reverse();

                        let args_shape: Vec<Type> = args.iter().map(|arg| arg.typ()).collect();
                        let result = self.run(i, args);

                        if self.should_jit {
                            self.jit.write_stats(i_usize, args_shape, result.typ());
                        }
                        result
                    };
                    #[cfg(target_arch = "wasm32")]
                    let result = {
                        let (argc, _, _) = &self.functions[i_usize];
                        let mut args = Vec::with_capacity(*argc as usize);

                        for _ in 0..*argc {
                            args.push(Self::pop_stack(&mut stack, &mut stack_pointer));
                        }

                        args.reverse();

                        let result = self.run(i, args);

                        result
                    };
                    Self::push_stack(&mut stack, &mut stack_pointer, result);
                }
                Bytecode::CallRet(i) => {
                    let (argc, localc, new_code) = self.borrow_function(i);

                    locals.clear();
                    for _ in 0..*argc {
                        locals.push(Self::pop_stack(&mut stack, &mut stack_pointer));
                    }
                    locals.reverse();

                    locals.extend(vec![Value::Int(0); *localc as usize]);

                    //println!("{:?}", locals);

                    code = new_code;
                    pc = 0;
                    continue;
                }
                Bytecode::Ret => return Self::pop_stack(&mut stack, &mut stack_pointer),
                Bytecode::Remove => {
                    Self::pop_stack(&mut stack, &mut stack_pointer);
                }
                Bytecode::NewPair => {
                    let right = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let left = Self::pop_stack(&mut stack, &mut stack_pointer);

                    Self::push_stack(
                        &mut stack,
                        &mut stack_pointer,
                        Value::Pair(Rc::new((left, right))),
                    );
                }
                Bytecode::PairLeft => {
                    let p = Self::pop_stack(&mut stack, &mut stack_pointer);

                    match p {
                        Value::Pair(p) => {
                            Self::push_stack(&mut stack, &mut stack_pointer, p.0.clone())
                        }
                        _ => Self::push_stack(&mut stack, &mut stack_pointer, Value::Nil),
                    }
                }
                Bytecode::PairRight => {
                    let p = Self::pop_stack(&mut stack, &mut stack_pointer);

                    match p {
                        Value::Pair(p) => {
                            Self::push_stack(&mut stack, &mut stack_pointer, p.1.clone())
                        }
                        _ => Self::push_stack(&mut stack, &mut stack_pointer, Value::Nil),
                    }
                }
                Bytecode::NewArray(s) => {
                    let s = s as usize;
                    let mut v = Vec::with_capacity(s);
                    for i in (0..s).rev() {
                        v[i] = Self::pop_stack(&mut stack, &mut stack_pointer);
                    }

                    Self::push_stack(
                        &mut stack,
                        &mut stack_pointer,
                        Value::Array(Rc::new(RefCell::new(v))),
                    );
                }
                Bytecode::NewArrayDyn => {
                    let size = Self::pop_stack(&mut stack, &mut stack_pointer);

                    if let Value::Int(size) = size {
                        Self::push_stack(
                            &mut stack,
                            &mut stack_pointer,
                            Value::Array(Rc::new(RefCell::new(vec![Value::Nil; size as usize]))),
                        );
                    } else {
                        vm_runtime_error!("{} must be an integer", size);
                    }
                }
                Bytecode::ArrayGet => {
                    let index = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let val = Self::pop_stack(&mut stack, &mut stack_pointer);

                    if let (Value::Array(arr), Value::Int(index)) = (&val, &index) {
                        Self::push_stack(
                            &mut stack,
                            &mut stack_pointer,
                            arr.borrow_mut()
                                .get(*index as usize)
                                .unwrap_or(&Value::Nil)
                                .clone(),
                        );
                    } else {
                        vm_runtime_error!(
                            "{} has to be array and {} has to be integer",
                            val,
                            index
                        );
                    }
                }
                Bytecode::ArraySet => {
                    let value = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let index = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let array = Self::pop_stack(&mut stack, &mut stack_pointer);

                    if let (Value::Array(arr), Value::Int(index)) = (&array, &index) {
                        arr.borrow_mut()[*index as usize] = value;
                        Self::push_stack(&mut stack, &mut stack_pointer, array);
                    } else {
                        vm_runtime_error!(
                            "{} has to be array and {} has to be integer",
                            array,
                            index
                        );
                    }
                }
                Bytecode::ArrayLen => {
                    let array = Self::pop_stack(&mut stack, &mut stack_pointer);

                    if let Value::Array(arr) = array {
                        Self::push_stack(
                            &mut stack,
                            &mut stack_pointer,
                            Value::Int(arr.borrow_mut().len() as i32),
                        )
                    } else {
                        vm_runtime_error!("{} is not array", array);
                    }
                }

                Bytecode::NewLambda(findex, ctx) => {
                    let s = ctx as usize;
                    let mut v = vec![Value::Nil; s];
                    for i in (0..s).rev() {
                        v[i] = Self::pop_stack(&mut stack, &mut stack_pointer);
                    }

                    Self::push_stack(
                        &mut stack,
                        &mut stack_pointer,
                        Value::Lambda(Rc::new((findex, v))),
                    );
                }
                Bytecode::CallLambda => {
                    let lambda = Self::pop_stack(&mut stack, &mut stack_pointer);

                    if let Value::Lambda(rc) = lambda {
                        let index = rc.0;
                        let ctx = &rc.1;

                        let (argc, _, _) = &self.functions[index as usize];
                        let mut args = Vec::with_capacity(*argc as usize);

                        for _ in 0..*argc {
                            args.push(Self::pop_stack(&mut stack, &mut stack_pointer));
                        }
                        args.push(Value::Array(Rc::new(RefCell::new(
                            ctx.iter().map(|v| v.clone()).collect(),
                        ))));

                        args.reverse();

                        Self::push_stack(&mut stack, &mut stack_pointer, self.run(index, args));
                    } else {
                        vm_runtime_error!("{} is not callable", lambda);
                    }
                }
                Bytecode::LocalArrayGet(local, index) => {
                    if let Value::Array(array) = &locals[local as usize] {
                        Self::push_stack(
                            &mut stack,
                            &mut stack_pointer,
                            array.borrow_mut()[index as usize].clone(),
                        )
                    } else {
                        vm_runtime_error!("{} is not an array", &locals[local as usize]);
                    }
                }
                Bytecode::Print => {
                    (self.print_handle)(&format!(
                        "{}",
                        Self::pop_stack(&mut stack, &mut stack_pointer)
                    ));
                    Self::push_stack(&mut stack, &mut stack_pointer, Value::Nil);
                }
                Bytecode::Atom(a) => {
                    Self::push_stack(&mut stack, &mut stack_pointer, Value::Atom(a))
                }
                Bytecode::IfEq => {
                    let left = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let right = Self::pop_stack(&mut stack, &mut stack_pointer);
                    if left != right {
                        let mut fc = 0;

                        loop {
                            pc += 1;
                            match code[pc] {
                                bc if bc.is_if() => fc += 1,
                                Bytecode::Else if fc == 0 => break,
                                Bytecode::Else if fc != 0 => fc -= 1,
                                _ => (),
                            }
                        }
                    }
                }
                Bytecode::IfLt => {
                    let right = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let left = Self::pop_stack(&mut stack, &mut stack_pointer);
                    if left < right {
                        let mut fc = 0;

                        loop {
                            pc += 1;
                            match code[pc] {
                                bc if bc.is_if() => fc += 1,
                                Bytecode::Else if fc == 0 => break,
                                Bytecode::Else if fc != 0 => fc -= 1,
                                _ => (),
                            }
                        }
                    }
                }
                Bytecode::IfGt => {
                    let right = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let left = Self::pop_stack(&mut stack, &mut stack_pointer);
                    if left > right {
                        let mut fc = 0;

                        loop {
                            pc += 1;
                            match code[pc] {
                                bc if bc.is_if() => fc += 1,
                                Bytecode::Else if fc == 0 => break,
                                Bytecode::Else if fc != 0 => fc -= 1,
                                _ => (),
                            }
                        }
                    }
                }
                Bytecode::IfLte => {
                    let right = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let left = Self::pop_stack(&mut stack, &mut stack_pointer);
                    if left <= right {
                        let mut fc = 0;

                        loop {
                            pc += 1;
                            match code[pc] {
                                bc if bc.is_if() => fc += 1,
                                Bytecode::Else if fc == 0 => break,
                                Bytecode::Else if fc != 0 => fc -= 1,
                                _ => (),
                            }
                        }
                    }
                }
                Bytecode::IfGte => {
                    let right = Self::pop_stack(&mut stack, &mut stack_pointer);
                    let left = Self::pop_stack(&mut stack, &mut stack_pointer);
                    if left >= right {
                        let mut fc = 0;

                        loop {
                            pc += 1;
                            match code[pc] {
                                bc if bc.is_if() => fc += 1,
                                Bytecode::Else if fc == 0 => break,
                                Bytecode::Else if fc != 0 => fc -= 1,
                                _ => (),
                            }
                        }
                    }
                }
                Bytecode::Nop => (),
            }
            pc += 1;
        }

        Value::Nil
    }

    #[inline(always)]
    fn push_stack(stack: &mut [Value], stack_pointer: &mut usize, val: Value) {
        stack[*stack_pointer] = val;
        *stack_pointer += 1;
    }

    #[inline(always)]
    fn pop_stack(stack: &mut [Value], stack_pointer: &mut usize) -> Value {
        *stack_pointer -= 1;
        let mut dummy = Value::Nil;
        std::mem::swap(&mut dummy, &mut stack[*stack_pointer]);
        return dummy;
    }

    #[inline(always)]
    fn borrow_function<'b>(&self, index: u32) -> &'b (u8, u8, Vec<Bytecode>) {
        unsafe { &*(&self.functions[index as usize] as *const _) }
    }
}
