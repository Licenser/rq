use std::collections::HashMap;

mod compiler;
mod expr;
mod jq;
mod parser;
mod std_lib;

use crate::compiler::*;
use crate::parser::*;
use clap::{App, Arg};
use jq::*;
use simd_json::{json, OwnedValue as Value};
use std::error::Error;
use std::io::{self, Read};
use std_lib::*;
use std::io::prelude::*;

#[no_mangle]
pub extern "C" fn printd(w: Wrap) {
    println!("{:?}", unsafe { &*w.json });
}

// Adding the functions above to a global array,
// so Rust compiler won't remove them.
#[used]
static EXTERNAL_FNS: [extern "C" fn(Wrap); 1] = [printd];

fn main() -> Result<(), Box<Error>> {
    let matches = App::new("rq")
        .version("1.0")
        .about("jq but compiled!")
        .author("Heinz G.")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .short("d")
                .help("Sets the input file to use")
                .required(false),
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let debug = matches.is_present("debug");
    let (_, p) = path(input).unwrap();
    let mut jq = Script::from_path(p);
    let jqs = jq.jit_compile_main(debug)?;

    for l in io::stdin().lock().lines() {
    unsafe {
        let mut l = l.unwrap();
    let mut json: Value = simd_json::to_owned_value(l.as_bytes_mut()).unwrap();
    let wrap = Wrap {
        error: 0,
        json: &json,
    };
        let r = jqs.call(wrap);
        if r.error == 0 {
            println!("{}", (&*r.json).to_string())
        } else {
            println!("Error: {}", r.error)
        }
    }
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
