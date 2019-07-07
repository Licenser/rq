use std::collections::HashMap;

mod compiler;
mod expr;
mod jq;
mod parser;
mod std_lib;

use crate::compiler::*;
use crate::parser::*;
use jq::*;
use std::error::Error;
use std_lib::*;

#[no_mangle]
pub extern "C" fn printd(w: Wrap) {
    println!("{:?}", unsafe { &*w.h });
}

// Adding the functions above to a global array,
// so Rust compiler won't remove them.
#[used]
static EXTERNAL_FNS: [extern "C" fn(Wrap); 1] = [printd];

fn main() -> Result<(), Box<Error>> {
    let mut hash: HashMap<String, String> = HashMap::new();
    hash.insert("a".into(), "b".into());
    let wrap = Wrap { h: &hash };
    /*
    let script = "let a = 3; let a = 4 + a; a * 6";
    let (r, ast) = exprs(script).expect("Unable build ast");
    dbg!(r);
    dbg!(&ast);

    let mut c = MathCompiler::new();
    let fun = c.jit_compile_expr_root(&ast)?;
    dbg!(unsafe { fun.call(wrap) });
    */

    let mut jq = Script::from_path(vec![]);
    let jqs = jq.jit_compile_main()?;

    let mut hash: HashMap<String, String> = HashMap::new();
    hash.insert("a".into(), "b".into());
    let wrap = Wrap { h: &hash };
    //    let res: Wrap = unsafe { *(jqs.call(wrap).as_ptr() as *const Wrap) };
    dbg!(unsafe { &*jqs.call(wrap).h });

    /*
    println!("Hello, world!");
    let _ = dbg!(path("."));
    let _ = dbg!(path(".[7]"));
    let _ = dbg!(path(".bla"));
    let _ = dbg!(path(".bla.blubb[7]"));
    */
    Ok(())
}
