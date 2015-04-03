#![feature(path_ext, exit_status)]

extern crate getopts;
extern crate rusqlite;
use std::env;
use std::io::prelude::*;
use std::io;
use std::path::Path;

static SCHEMA: &'static str = include_str!("../gps-db.sql");

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this message");
    opts.optflag("f", "force", "overwrite the database if it already exists");

    let matches = match opts.parse(args) {
        Ok(m) => m,
        Err(e) => {
            panic!("invalid arguments: {}", e)
        }
    };

    if matches.opt_present("h") || matches.free.len() != 1 {
        println!("{}", opts.usage(&format!("Usage: {} [options] DATABASE", program)));
        return
    }

    let file = &matches.free[0];
    let path = Path::new(file);
    if path.exists() && !matches.opt_present("f") {
        writeln!(&mut io::stderr(), "error: file `{}` already exists", file).unwrap();
        env::set_exit_status(1);
        return
    }

    let conn = match rusqlite::SqliteConnection::open(path) {
        Ok(c) => c,
        Err(e) => {
            writeln!(&mut io::stderr(), "error: could not open `{}`: {}",  file, e).unwrap();
            env::set_exit_status(2);
            return
        }
    };
    match conn.execute_batch(SCHEMA) {
        Ok(_) => {},
        Err(e) => {
            writeln!(&mut io::stderr(), "error: failed to execute schema: {}", e).unwrap();
            env::set_exit_status(3);
            return
        }
    }
}
