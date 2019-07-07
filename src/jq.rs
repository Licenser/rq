use crate::compiler::{Compile, Compiler, CompilerError, Wrap};
use crate::STDLIB;

use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{FunctionValue, PointerValue, StructValue};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;

#[derive(Debug, Clone)]
pub enum Path {
    Root,
    Key(String),
    Idx(usize),
}

pub trait JQCompile<Ret, Comp: Compiler> {
    fn compile(&self, compiler: &Comp, val: StructValue) -> Result<Ret, CompilerError>;
}

impl JQCompile<StructValue, Script> for Path {
    fn compile(&self, compiler: &Script, val: StructValue) -> Result<StructValue, CompilerError> {
        match self {
            Path::Root => Ok(val),
            Path::Key(k) => {
                let i64_type = compiler.context().i64_type();
                let key_str = compiler.builder().build_global_string_ptr(&k, "key");
                let key_len = i64_type.const_int(k.len() as u64, false);
                let fun = compiler.get_function("jq_get_key")?;
                let r = compiler
                    .builder()
                    .build_call(
                        fun,
                        &[
                            val.into(),
                            key_str.as_pointer_value().into(),
                            key_len.into(),
                        ],
                        "get",
                    )
                    .try_as_basic_value()
                    .left();
                match r {
                    Some(s) => Ok(s.into_struct_value()),
                    None => Err(CompilerError::Generic),
                }
            }
            Path::Idx(idx) => {
                let i64_type = compiler.context().i64_type();
                let idx = i64_type.const_int(*idx as u64, false);
                let fun = compiler.get_function("jq_get_idx")?;
                let r = compiler
                    .builder()
                    .build_call(
                        fun,
                        &[
                            val.into(),
                            idx.into(),
                        ],
                        "get",
                    )
                    .try_as_basic_value()
                    .left();
                match r {
                    Some(s) => Ok(s.into_struct_value()),
                    None => Err(CompilerError::Generic),
                }
            }
            _ => Err(CompilerError::Generic),
        }
    }
}

type MainFunc = unsafe extern "C" fn(Wrap) -> Wrap;

pub struct Script {
    pub script: Vec<Path>,
    pub context: Context,
    pub module: Module,
    pub builder: Builder,
    pub execution_engine: ExecutionEngine,
    pub variables: HashMap<String, PointerValue>,
    pub fn_value_opt: Option<FunctionValue>,
    pub json_struct: StructType,
}

impl Compiler for Script {
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

impl Script {
    pub fn from_path(script: Vec<Path>) -> Self {
        let context = Context::create();
        let module = context.create_module("jq");
        let builder = context.create_builder();
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();
        let i64_type = context.i64_type();
        let json_struct = context.struct_type(&[i64_type.into(), i64_type.into()], false);

        let compiler = Self {
            context,
            module,
            builder,
            execution_engine,
            variables: HashMap::new(),
            fn_value_opt: None,
            json_struct,
            script,
        };
        for p in &STDLIB {
            p.compile::<Script>(&compiler);
        }
        compiler
    }

    pub fn jit_compile_main(&mut self, debug: bool) -> Result<JitFunction<MainFunc>, CompilerError> {
        let ret_type = self.json_struct;

        let fn_type = ret_type.fn_type(&[self.json_struct().into()], false);
        let function = self.module.add_function("main", fn_type, None);
        let w = function.get_nth_param(0).unwrap().into_struct_value();

        self.fn_value_opt = Some(function);
        let basic_block = self.context.append_basic_block(&function, "entry");
        self.builder.position_at_end(&basic_block);

        let mut res = w;
        for expr in &self.script {
            res = expr.compile(self, res)?
        }

        self.builder.build_return(Some(&res));

    if debug{
        self.module.print_to_stderr();}
        unsafe {
            self.execution_engine
                .get_function("main")
                .map_err(|_| CompilerError::Generic)
        }
    }
}
