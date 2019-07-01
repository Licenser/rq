use crate::expr::*;
use std::fmt;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::OptimizationLevel;

pub struct Wrap {
    pub h: *const HashMap<String, String>,
}

pub trait Compile {
    fn compile(&self, compiler: &mut Compiler) -> Result<IntValue, CompilerError>;
}

#[derive(Debug)]
pub enum CompilerError {
    Generic,
    UnknownVariable(String),
}
impl Error for CompilerError {}

impl Display for CompilerError {
    fn fmt(&self, format: &mut Formatter) -> fmt::Result {
        write!(format, "{:?}", self)
    }
}

type MainFunc = unsafe extern "C" fn(Wrap) -> i64;

pub struct Compiler {
    pub context: Context,
    pub module: Module,
    pub builder: Builder,
    pub execution_engine: ExecutionEngine,
    pub variables: HashMap<String, PointerValue>,
    pub fn_value_opt: Option<FunctionValue>,
    pub wrap_struct: StructType,
}

/// Defines the prototype (name and parameters) of a function.
#[derive(Debug)]
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>,
    //pub is_op: bool,
    //pub prec: usize
}

impl Compiler {
    pub fn new() -> Self {
        let context = Context::create();
        let module = context.create_module("toylang");
        let builder = context.create_builder();
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();
        let i64_type = context.i64_type();
        let wrap_struct = context.struct_type(&[i64_type.into()], false);

        Self {
            context,
            module,
            builder,
            execution_engine,
            variables: HashMap::new(),
            fn_value_opt: None,
            wrap_struct,
        }
    }
    #[inline]
    pub fn get_function(&self, name: &str) -> Option<FunctionValue> {
        self.module.get_function(name)
    }
    pub fn init(&self, ps: &[Prototype]) -> &Self {
        for p in ps {
            self.compile_prototype(p);
        }
        self
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

    fn compile_prototype(&self, proto: &Prototype) -> Result<FunctionValue, CompilerError> {
        let ret_type = self.context.i64_type();
        let args_types: Vec<BasicTypeEnum> = vec![self.wrap_struct.into(), ret_type.into()];
        let args_types = args_types.as_slice();

        let fn_type = self.context.i64_type().fn_type(args_types, false);
        let fn_val = self.module.add_function(proto.name.as_str(), fn_type, None);
        // set arguments names
        for (i, arg) in fn_val.get_param_iter().enumerate() {
            if i == 0 {
                arg.into_struct_value().set_name(proto.args[i].as_str());
            } else {
                arg.into_int_value().set_name(proto.args[i].as_str());
            }
        }
        // finally return built prototype
        Ok(fn_val)
    }

    pub fn jit_compile_expr_root(
        &mut self,
        exprs: &[Expr],
    ) -> Result<JitFunction<MainFunc>, CompilerError> {
        let i64_type = self.context.i64_type();

        let fn_type = i64_type.fn_type(&[self.wrap_struct.into()], false);
        let function = self.module.add_function("main", fn_type, None);
        let w = function.get_nth_param(0).unwrap().into_struct_value();

        self.fn_value_opt = Some(function);
        let basic_block = self.context.append_basic_block(&function, "entry");
        self.builder.position_at_end(&basic_block);

        let mut res = i64_type.const_int(0, false);
        for expr in exprs {
            res = expr.compile(self)?
        }

        let fun = self.module.get_function("printd").unwrap();
        dbg!(&fun);
        let res = match self
            .builder
            .build_call(fun, &[w.into(), res.into()], "call")
            .try_as_basic_value()
            .left()
        {
            Some(value) => value.into_int_value(),
            None => {
                return Err(CompilerError::Generic);
            }
        };

        self.builder.build_return(Some(&res));

        self.module.print_to_stderr();
        unsafe {
            self.execution_engine
                .get_function("main")
                .map_err(|_| CompilerError::Generic)
        }
    }
}
