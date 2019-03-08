extern crate winapi;

mod cheat;
mod util;

fn main() { { if let Err(e) = cheat::option() { println!("{:?}", e); } } }
