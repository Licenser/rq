use std::collections::HashMap;

mod compiler;
mod expr;
mod parser;

use crate::compiler::*;

use crate::parser::*;

use std::error::Error;

#[no_mangle]
pub extern "C" fn printd(w: Wrap, x: i64) -> i64 {
    println!("==> {} / {:?}", x, unsafe{&*w.h});
    x
}

// Adding the functions above to a global array,
// so Rust compiler won't remove them.
#[used]
static EXTERNAL_FNS: [extern "C" fn(Wrap, i64) -> i64; 1] = [printd];

fn main() -> Result<(), Box<Error>> {
    let mut hash: HashMap<String, String> = HashMap::new();
     hash.insert("a".into(), "b".into());
    let wrap = Wrap { h: &hash };
    let script = "let a = 3; let a = 4 + a; a * 6";
    let (r, ast) = exprs(script).expect("Unable build ast");
    dbg!(r);
    dbg!(&ast);

    let mut c = Compiler::new();
    c.init(&[Prototype {
        name: "printd".to_string(),
        args: vec!["w".to_string(), "x".to_string()],
    }]);
    let fun = c.jit_compile_expr_root(&ast)?;
    dbg!(unsafe { fun.call(wrap) });
    println!("Hello, world!");
    Ok(())
}
