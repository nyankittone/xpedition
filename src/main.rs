extern crate x11rb;

use std::time;
use std::fmt::{self, Display, Debug};
use std::error::Error;

use x11rb::connection::Connection;
use x11rb::errors::{ConnectionError, ReplyError, ReplyOrIdError};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::COPY_DEPTH_FROM_PARENT;
// use x11rb_protocol::SequenceNumber;

#[derive(Debug)]
pub struct SimpleError<T: Display + Debug> {
    value: T,
}

impl<T: Display + Debug> Display for SimpleError<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.value)
    }
}

impl<T: Display + Debug> Error for SimpleError<T> {}

impl<T: Display + Debug> SimpleError<T> {
    pub const fn new(thing: T) -> Self {
        Self {
            value: thing,
        }
    }
}

impl<T: Display + Debug> From<T> for SimpleError<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

fn bad_async(display: &RustConnection, atoms: &mut [u32], atom_names: &[impl AsRef<str>])
-> Result<time::Duration, Box<dyn std::error::Error>> {
    if atoms.len() != atom_names.len() {
        return Err("Mismatched lengths of inputted slices".into());
    }

    let start = time::Instant::now();
    for i in 0..atoms.len() {
        atoms[i] = display.intern_atom(false, atom_names[i].as_ref().as_bytes())?.reply()?.atom;
    }

    Ok(start.elapsed())
}

fn good_async(display: &RustConnection, atoms: &mut [u32], atom_names: &[impl AsRef<str>])
-> Result<time::Duration, Box<dyn std::error::Error>> {
    if atoms.len() != atom_names.len() {
        return Err("Mismatched lengths of inputted slices".into());
    }

    let start = time::Instant::now();

    let mut cookies = Vec::new();
    cookies.reserve_exact(atoms.len());
    cookies.extend(atom_names.iter().map(|name| display.intern_atom(false, name.as_ref().as_bytes())));

    for (i, atom) in cookies.into_iter().enumerate() {
        atoms[i] = atom?.reply()?.atom;
    }

    Ok(start.elapsed())
}

fn main() {
    const ATOM_COUNT: usize = 512;

    let (display, _) = x11rb::connect(None).unwrap();
    let mut atoms = [Into::<u32>::into(AtomEnum::NONE); ATOM_COUNT];
    let atom_names: [String; ATOM_COUNT] = std::array::from_fn(|n| format!("NAME{n}"));

    let bad_elapsed = bad_async(&display, &mut atoms, &atom_names).unwrap();
    let good_elapsed = good_async(&display, &mut atoms, &atom_names).unwrap();

    println! (
        "bad time: {} ns\ngood time: {} ns\nbad/good ratio: {}",
        bad_elapsed.as_nanos(),
        good_elapsed.as_nanos(),
        bad_elapsed.as_nanos() as f64 / good_elapsed.as_nanos() as f64
    );
}
