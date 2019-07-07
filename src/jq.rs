use crate::compiler::{Compile, Compiler, CompilerError, Wrap};

use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::OptimizationLevel;

#[derive(Debug)]
pub enum Path {
    Root,
    Key(String),
    Idx(usize),
}

type MainFunc = unsafe extern "C" fn(Wrap) -> i64;


pub struct Script {
    pub script: Vec<Path>,
    pub context: Context,
    pub module: Module,
    pub builder: Builder,
    pub execution_engine: ExecutionEngine,
    pub variables: HashMap<String, PointerValue>,
    pub fn_value_opt: Option<FunctionValue>,
    pub wrap_struct: StructType,}



impl Script {
    fn from_path(script :Vec<Path>) -> Self {
        let context = Context::create();
        let module = context.create_module("jq");
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
            script
        }
    }

    pub fn jit_compile_main(
        &mut self,
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

    fn compile(&self, compiler: &mut Compiler) -> Result<PointerValue, CompilerError> {
        let l = self.left.compile(compiler)?;
        let r = self.right.compile(compiler)?;
        Ok(compiler.builder.build_int_add(l, r, "add"))
    }
}