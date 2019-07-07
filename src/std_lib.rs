use crate::compiler::*;

use inkwell::types::BasicTypeEnum;
use inkwell::values::FunctionValue;

#[derive(Debug, Clone, Copy)]
pub enum JQType {
    JSON,
    //    Stirng,
    Integer,
    Float,
    Void,
    //    Array
}

/// Defines the prototype (name and parameters) of a function.
#[derive(Debug)]
pub struct Prototype {
    pub name: &'static str,
    pub args: &'static [(&'static str, JQType)],
    pub ret: JQType,
}
impl Prototype {
    pub fn compile<C: Compiler>(&self, compiler: &Compiler) -> Result<FunctionValue, CompilerError> {

        let args_types: Vec<BasicTypeEnum> = self
            .args
            .iter()
            .map(|(_n, t)| compiler.type_for(t))
            .collect();
        let args_types = args_types.as_slice();

        let fn_type = compiler.fn_type_for(&self.ret, args_types);
        let fn_val = compiler.module().add_function(self.name, fn_type, None);

        // finally return built prototype
        Ok(fn_val)
    }
}

pub static STDLIB: [Prototype; 3] = [
    Prototype {
        name: "printd",
        args: &[("w", JQType::JSON)],
        ret: JQType::Void,
    },
    Prototype {
        name: "printjson",
        args: &[("w", JQType::JSON)],
        ret: JQType::Void,
    },
    Prototype {
        name: "dbg",
        args: &[],
        ret: JQType::Void,
    },
];

#[used]
static E_PRINTJSON: [extern "C" fn(Wrap); 1] = [printjson];
#[no_mangle]
pub extern "C" fn printjson(w: Wrap) {
    println!("{:?}", unsafe { &*w.h });
}

#[used]
static E_DBG: [extern "C" fn(); 1] = [dbg];
#[no_mangle]
pub extern "C" fn dbg() {
    dbg!();
}

#[used]
static E_GET: extern "C" fn(Wrap, *const u8, u8) -> Wrap = jq_get;
#[no_mangle]
pub extern "C" fn jq_get(w: Wrap, key: *const u8, len: u8) -> Wrap {
    println!("==> {:?}", unsafe { &*w.h });
    w
}
