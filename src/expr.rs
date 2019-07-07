use crate::compiler::*;
use inkwell::values::IntValue;
use std::fmt::{self, Debug, Display, Formatter};

//#[derive(Debug)]
pub enum Expr {
    Value(i64),
    Add(AddExpr),
    Sub(SubExpr),
    Mul(MulExpr),
    Div(DivExpr),
    Var(String),
    Let(String, Box<Expr>),
    Paren(Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, format: &mut Formatter) -> fmt::Result {
        use self::Expr::*;
        match *self {
            Value(ref val) => write!(format, "{}", val),
            Add(ref e) => write!(format, "{} + {}", e.left, e.right),
            Sub(ref e) => write!(format, "{} - {}", e.left, e.right),
            Mul(ref e) => write!(format, "{} * {}", e.left, e.right),
            Div(ref e) => write!(format, "{} / {}", e.left, e.right),
            Var(ref v) => write!(format, "{}", v),
            Let(ref v, ref right) => write!(format, "let {} = {}", v, right),
            Paren(ref expr) => write!(format, "({})", expr),
        }
    }
}

impl Debug for Expr {
    fn fmt(&self, format: &mut Formatter) -> fmt::Result {
        use self::Expr::*;
        match *self {
            Value(ref val) => write!(format, "{}", val),
            Add(ref e) => write!(format, "({:?} + {:?})", e.left, e.right),
            Sub(ref e) => write!(format, "({:?} - {:?})", e.left, e.right),
            Mul(ref e) => write!(format, "({:?} * {:?})", e.left, e.right),
            Div(ref e) => write!(format, "({:?} / {:?})", e.left, e.right),
            Var(ref v) => write!(format, "{:?}", v),
            Let(ref v, ref right) => write!(format, "let {:?} = {:?}", v, right),
            Paren(ref expr) => write!(format, "[{:?}]", expr),
        }
    }
}

pub struct AddExpr {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl Compile<IntValue, MathCompiler> for AddExpr {
    fn compile(&self, compiler: &mut MathCompiler) -> Result<IntValue, CompilerError> {
        let l = self.left.compile(compiler)?;
        let r = self.right.compile(compiler)?;
        Ok(compiler.builder.build_int_add(l, r, "add"))
    }
}

pub struct SubExpr {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl Compile<IntValue, MathCompiler> for SubExpr {
    fn compile(&self, compiler: &mut MathCompiler) -> Result<IntValue, CompilerError> {
        let l = self.left.compile(compiler)?;
        let r = self.right.compile(compiler)?;
        Ok(compiler.builder.build_int_sub(l, r, "add"))
    }
}

pub struct MulExpr {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl Compile<IntValue, MathCompiler> for MulExpr {
    fn compile(&self, compiler: &mut MathCompiler) -> Result<IntValue, CompilerError> {
        let l = self.left.compile(compiler)?;
        let r = self.right.compile(compiler)?;
        Ok(compiler.builder.build_int_mul(l, r, "add"))
    }
}

pub struct DivExpr {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl Compile<IntValue, MathCompiler> for DivExpr {
    fn compile(&self, compiler: &mut MathCompiler) -> Result<IntValue, CompilerError> {
        let l = self.left.compile(compiler)?;
        let r = self.right.compile(compiler)?;
        Ok(compiler.builder().build_int_signed_div(l, r, "add"))
    }
}
impl Compile<IntValue, MathCompiler> for Expr {
    fn compile(&self, compiler: &mut MathCompiler) -> Result<IntValue, CompilerError> {
        let i64_type = compiler.context().i64_type();
        match self {
            Expr::Value(v) => Ok(if *v < 0 {
                i64_type.const_int((*v * -1) as u64, true)
            } else {
                i64_type.const_int(*v as u64, false)
            }),
            Expr::Add(e) => e.compile(compiler),
            Expr::Sub(e) => e.compile(compiler),
            Expr::Mul(e) => e.compile(compiler),
            Expr::Div(e) => e.compile(compiler),
            Expr::Paren(e) => e.compile(compiler),
            Expr::Var(ref name) => match compiler.variables.get(name.as_str()) {
                Some(var) => Ok(compiler
                    .builder
                    .build_load(*var, name.as_str())
                    .into_int_value()),
                None => Err(CompilerError::UnknownVariable(name.to_string())),
            },
            Expr::Let(ref name, ref e) => {
                let var_name = name.as_str();
                let alloca = compiler.create_entry_block_alloca(var_name, None);
                let val = e.compile(compiler)?;
                compiler.builder.build_store(alloca, val);
                compiler.variables.insert(var_name.to_string(), alloca);
                Ok(val)
            }
            _ => Ok(i64_type.const_int(0, false)),
        }
    }
}
