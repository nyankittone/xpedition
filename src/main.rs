extern crate x11rb;

use std::{time, thread};

use x11rb::connection::Connection;
use x11rb::errors::{ConnectionError, ReplyError, ReplyOrIdError};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::COPY_DEPTH_FROM_PARENT;
// use x11rb_protocol::SequenceNumber;

fn main() {
    let (display, screen_id) = x11rb::connect(None).unwrap();
    let screen = &display.setup().roots[screen_id];

    let window = display.generate_id().unwrap();
    display.create_window (
        COPY_DEPTH_FROM_PARENT,
        window,
        screen.root,
        0,
        0,
        800,
        600,
        1,
        WindowClass::INPUT_OUTPUT,
        screen.root_visual,
        &Default::default(),
    ).unwrap();

    display.map_window(window).unwrap();
    display.sync().unwrap();

    thread::sleep(time::Duration::from_secs(5));
}
