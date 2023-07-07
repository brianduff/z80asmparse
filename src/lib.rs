use pest_derive::Parser;

pub mod ast;

#[derive(Parser)]
#[grammar = "z80asm.pest"]
pub struct Z80Parser;

