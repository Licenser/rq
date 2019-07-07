use std::collections::HashMap;

mod compiler;
mod expr;
mod jq;
mod parser;
mod std_lib;

use crate::compiler::*;
use crate::parser::*;
use jq::*;
use simd_json::{json, OwnedValue as Value};
use std::error::Error;
use std_lib::*;

#[no_mangle]
pub extern "C" fn printd(w: Wrap) {
    println!("{:?}", unsafe { &*w.json });
}

// Adding the functions above to a global array,
// so Rust compiler won't remove them.
#[used]
static EXTERNAL_FNS: [extern "C" fn(Wrap); 1] = [printd];

fn main() -> Result<(), Box<Error>> {
    let mut json: Value = json!({
        "a": [1, 2, 3],
        "b": {"c": "d"}
    });
    let wrap = Wrap {
        error: 0,
        json: &json,
    };

    let mut jq = Script::from_path(vec![Path::Root, Path::Key("a".into()), Path::Idx(1)]);
    let jqs = jq.jit_compile_main()?;
    let r = unsafe { jqs.call(wrap) };
    dbg!(unsafe { &*r.json });
    dbg!(unsafe { r.error });
    if r.error == 0 { 
        println!((&*r.json).to_string())
    } else {
        println!("Error: {}", r.error)
    }

    /*
    let script = "let a = 3; let a = 4 + a; a * 6";
    let (r, ast) = exprs(script).expect("Unable build ast");
    dbg!(r);
    dbg!(&ast);

    let mut c = MathCompiler::new();
    let fun = c.jit_compile_expr_root(&ast)?;
    dbg!(unsafe { fun.call(wrap) });
    println!("Hello, world!");
    let _ = dbg!(path("."));
    let _ = dbg!(path(".[7]"));
    let _ = dbg!(path(".bla"));
    let _ = dbg!(path(".bla.blubb[7]"));
    */
    Ok(())
}
