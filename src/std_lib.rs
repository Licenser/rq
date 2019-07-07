use crate::compiler::*;

use inkwell::types::BasicTypeEnum;
use inkwell::values::FunctionValue;
use simd_json::value::ValueTrait;

#[derive(Debug, Clone, Copy)]
pub enum JQType {
    JSON,
    String,
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
    pub fn compile<C: Compiler>(
        &self,
        compiler: &Compiler,
    ) -> Result<FunctionValue, CompilerError> {
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

pub static STDLIB: [Prototype; 5] = [
    Prototype {
        name: "printd",
        args: &[("json", JQType::JSON)],
        ret: JQType::Void,
    },
    Prototype {
        name: "printjson",
        args: &[("json", JQType::JSON)],
        ret: JQType::Void,
    },
    Prototype {
        name: "dbg",
        args: &[],
        ret: JQType::Void,
    },
    Prototype {
        name: "jq_get_key",
        args: &[
            ("json", JQType::JSON),
            ("key", JQType::String),
            ("len", JQType::Integer),
        ],
        ret: JQType::JSON,
    },
    Prototype {
        name: "jq_get_idx",
        args: &[("json", JQType::JSON), ("idx", JQType::Integer)],
        ret: JQType::JSON,
    },
];

#[used]
static E_PRINTJSON: [extern "C" fn(Wrap); 1] = [printjson];
#[no_mangle]
pub extern "C" fn printjson(w: Wrap) {
    println!("{:?}", unsafe { &*w.json });
}

#[used]
static E_DBG: [extern "C" fn(); 1] = [dbg];
#[no_mangle]
pub extern "C" fn dbg() {
    dbg!();
}

#[used]
static E_GET_KEY: unsafe extern "C" fn(Wrap, *const u8, usize) -> Wrap = jq_get_key;
#[no_mangle]
pub unsafe extern "C" fn jq_get_key(mut wrap: Wrap, key: *const u8, len: usize) -> Wrap {
    use std::slice::from_raw_parts;
    use std::str;
    let key_slice: &[u8] = from_raw_parts(key, len);
    let key_str = str::from_utf8(key_slice).unwrap();
    if let Some(o) = (&*wrap.json).as_object() {
        if let Some(v) = o.get(key_str) {
            wrap.json = v;
        } else {
            wrap.error = 1;
        }
    } else {
        wrap.error = 2;
    }
    wrap
}

#[used]
static E_GET_IDX: unsafe extern "C" fn(Wrap, usize) -> Wrap = jq_get_idx;
#[no_mangle]
pub unsafe extern "C" fn jq_get_idx(mut wrap: Wrap, idx: usize) -> Wrap {
    if let Some(o) = (&*wrap.json).as_array() {
        if let Some(v) = o.get(idx) {
            wrap.json = v;
        } else {
            wrap.error = 1;
        }
    } else {
        wrap.error = 2;
    }
    wrap
}
