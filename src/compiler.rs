use crate::expr::*;
use crate::std_lib::*;

use simd_json::OwnedValue as Value;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, FunctionType, StructType};
use inkwell::values::{FunctionValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;

#[derive(Debug)]
pub struct Wrap {
    pub error: u64,
    pub json: *const Value,
}

pub trait Compile<Ret, Comp: Compiler> {
    fn compile(&self, compiler: &mut Comp) -> Result<Ret, CompilerError>;
}

pub trait Compiler {
    fn context(&self) -> &Context;
    fn module(&self) -> &Module;
    fn json_struct(&self) -> StructType;
    fn builder(&self) -> &Builder;
    fn type_for(&self, t: &JQType) -> BasicTypeEnum {
        match t {
            JQType::JSON => self.json_struct().into(),
            JQType::Integer => self.context().i64_type().into(),
            JQType::Float => self.context().f64_type().into(),
            JQType::String => self
                .context()
                .i8_type()
                .ptr_type(AddressSpace::Global)
                .into(),
            JQType::Void => unreachable!(),
        }
    }
    fn fn_type_for(&self, t: &JQType, args: &[BasicTypeEnum]) -> FunctionType {
        match t {
            JQType::JSON => self.json_struct().fn_type(args, false),
            JQType::Integer => self.context().i64_type().fn_type(args, false),
            JQType::Float => self.context().f64_type().fn_type(args, false),
            JQType::Void => self.context().void_type().fn_type(args, false),
            JQType::String => unreachable!(),
        }
    }
    #[inline]
    fn get_function(&self, name: &str) -> Result<FunctionValue, CompilerError> {
        match self.module().get_function(name) {
            Some(f) => Ok(f),
            None => Err(CompilerError::UnknownFunction(name.to_string())),
        }
    }
}

impl Compiler for MathCompiler {
    fn context(&self) -> &Context {
        &self.context
    }
    fn module(&self) -> &Module {
        &self.module
    }
    fn builder(&self) -> &Builder {
        &self.builder
    }
    fn json_struct(&self) -> StructType {
        self.json_struct
    }
}

#[derive(Debug)]
pub enum CompilerError {
    Generic,
    UnknownFunction(String),
    UnknownVariable(String),
}
impl Error for CompilerError {}

impl Display for CompilerError {
    fn fmt(&self, format: &mut Formatter) -> fmt::Result {
        write!(format, "{:?}", self)
    }
}

type MainFunc = unsafe extern "C" fn(Wrap) -> i64;

pub struct MathCompiler {
    pub context: Context,
    pub module: Module,
    pub builder: Builder,
    pub execution_engine: ExecutionEngine,
    pub variables: HashMap<String, PointerValue>,
    pub fn_value_opt: Option<FunctionValue>,
    pub json_struct: StructType,
}

impl MathCompiler {
    pub fn new() -> Self {
        let context = Context::create();
        let module = context.create_module("toylang");
        let builder = context.create_builder();
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();
        let i64_type = context.i64_type();
        let json_struct = context.struct_type(&[i64_type.into(), i64_type.into()], false);

        let c = Self {
            context,
            module,
            builder,
            execution_engine,
            variables: HashMap::new(),
            fn_value_opt: None,
            json_struct,
        };
        for p in &STDLIB {
            p.compile::<MathCompiler>(&c);
        }
        c
    }

    /// Returns the `FunctionValue` representing the function being compiled.
    #[inline]
    fn fn_value(&self) -> FunctionValue {
        self.fn_value_opt.unwrap()
    }

    pub fn create_entry_block_alloca(
        &self,
        name: &str,
        entry: Option<&BasicBlock>,
    ) -> PointerValue {
        let builder = self.context.create_builder();

        let owned_entry = self.fn_value().get_entry_basic_block();
        let entry = owned_entry.as_ref().or(entry).unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.i64_type(), name)
    }

    pub fn jit_compile_expr_root(
        &mut self,
        exprs: &[Expr],
    ) -> Result<JitFunction<MainFunc>, CompilerError> {
        let i64_type = self.context.i64_type();

        let fn_type = i64_type.fn_type(&[self.json_struct.into()], false);
        let function = self.module.add_function("main", fn_type, None);
        let w = function.get_nth_param(0).unwrap().into_struct_value();

        self.fn_value_opt = Some(function);
        let basic_block = self.context.append_basic_block(&function, "entry");
        self.builder.position_at_end(&basic_block);

        let mut res = i64_type.const_int(0, false);
        for expr in exprs {
            res = expr.compile(self)?
        }

        let fun = self.get_function("printjson")?;
        dbg!(&fun);
        self.builder.build_call(fun, &[], "call");

        self.builder.build_return(Some(&res));

        self.module.print_to_stderr();
        unsafe {
            self.execution_engine
                .get_function("main")
                .map_err(|_| CompilerError::UnknownFunction("main".to_string()))
        }
    }
}
