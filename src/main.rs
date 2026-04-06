extern crate x11rb;

use std::{time, thread};

use x11rb::connection::Connection;
use x11rb::errors::{ConnectionError, ReplyError, ReplyOrIdError};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::COPY_DEPTH_FROM_PARENT;

// program idea: draw a path controlled by WASD, and have it never be cleared, and have the path
// color change over time.

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
        &CreateWindowAux {
            background_pixel: Some(screen.black_pixel),
            event_mask: Some(EventMask::EXPOSURE),
            ..Default::default()
        },
    ).unwrap();

    // Window attributes?

    // Create a graphics context???
    let gc = display.generate_id().unwrap();
    display.create_gc(
        gc, 
        screen.root,
        &CreateGCAux::default().foreground(screen.white_pixel),
    ).unwrap();

    display.map_window(window).unwrap();
    display.flush().unwrap();

    // How to draw shapes????
    loop {
        match display.wait_for_event().unwrap() {
            Event::Expose(_) => {
                display.poly_fill_rectangle(window, gc, &[Rectangle {
                    x: 300,
                    y: 300,
                    width: 60,
                    height: 80,
                }]).unwrap();
            },
            _ => (),
        }

        display.flush().unwrap();
    }
}
