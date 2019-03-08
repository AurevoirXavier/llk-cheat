extern crate winapi;

mod cheat;
mod util;

fn main() { loop { if let Err(e) = cheat::option() { println!("{:?}", e); } } }
