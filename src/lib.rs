extern crate indexmap;

pub mod plurality;
pub mod stv;



pub type Selection<'a> = (&'a str, u32);
pub type Selections<'a> = Vec<Selection<'a>>;

