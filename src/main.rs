extern crate x11rb;

use std::{time, thread};
use std::fmt::{self, Display, Debug};

use x11rb::connection::Connection;
use x11rb::errors::{ConnectionError, ReplyError, ReplyOrIdError};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::COPY_DEPTH_FROM_PARENT;

// program idea: draw a path controlled by WASD, and have it never be cleared, and have the path
// color change over time.

// This type conceptually might be dumb.
#[derive(Debug)]
pub enum XError {
    Connection(ConnectionError),
    ReplyOrId(ReplyOrIdError),
}

impl Display for XError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(error) => write!(formatter, "Connection error: {}", error)?,
            Self::ReplyOrId(error) => write!(formatter, "Reply or ID error: {}", error)?,
        }

        Ok(())
    }
}

impl std::error::Error for XError {}

impl From<ConnectionError> for XError {
    fn from(value: ConnectionError) -> Self {
        Self::Connection(value)
    }
}

impl From<ReplyOrIdError> for XError {
    fn from(value: ReplyOrIdError) -> Self {
        Self::ReplyOrId(value)
    }
}

fn while_connected(display: &impl Connection, screen: &Screen) -> Result<(), XError> {
    const WIDTH: u16 = 800;
    const HEIGHT: u16 = 600;
    const LINE_THICKNESS: u32 = 4;

    // create an appropriate GC and window
    // generate XIDs for window and GC
    let (window, gc) = (
        display.generate_id().map_err(|e| XError::from(e))?,
        display.generate_id().map_err(|e| XError::from(e))?,
    );

    let delete_signal = display.intern_atom(false, b"WM_DELETE_WINDOW")?;
    let wm_protocols = display.intern_atom(false, b"WM_PROTOCOLS")?;

    // create GC and window
    display.create_gc (
        gc,
        screen.root,
        &CreateGCAux::default().foreground(screen.white_pixel).line_width(LINE_THICKNESS)
    )?;

    display.create_window (
        COPY_DEPTH_FROM_PARENT,
        window,
        screen.root,
        0,
        0,
        WIDTH,
        HEIGHT,
        0,
        WindowClass::INPUT_OUTPUT,
        screen.root_visual, &CreateWindowAux::default()
            .backing_pixel(screen.black_pixel)
            .event_mask(EventMask::EXPOSURE)
    )?;

    // map the window
    display.map_window(window)?;

    // await the WM_DELETE_WINDOW atom and add it to WM_PROTOCOLS on our window
    let delete_signal = delete_signal.reply().map_err(|e| ReplyOrIdError::from(e))?.atom;
    let wm_protocols = wm_protocols.reply().map_err(|e| ReplyOrIdError::from(e))?.atom;
    display.change_property32 (
        PropMode::REPLACE,
        window,
        wm_protocols,
        AtomEnum::ATOM,
        &[delete_signal]
    )?;

    display.flush()?;

    // in a loop, handle keyboard inputs, Expose, and WM_DELETE_WINDOW
    // while also just drawing like normal.
    #[allow(unreachable_code)]
    Ok(loop {
        match display.wait_for_event()? {
            Event::Expose(_) => {
                display.poly_line (
                    CoordMode::ORIGIN,
                    window,
                    gc,
                    &[Point{x: 100, y: 100}, Point{x: 200, y: 150}, Point{x: 250, y: 250}]
                )?;
            },
            Event::ClientMessage(ClientMessageEvent {format: 32, data, ..}) => {
                if data.as_data32()[0] == delete_signal {
                    return Ok(());
                }
            },
            _ => (),
        }

        display.flush()?;
    })
}

fn main() {
    let (display, screen) = x11rb::connect(None).unwrap();
    let screen = &display.setup().roots[screen];
    while_connected(&display, screen).unwrap();

    println!("Goodbye!!~ >w<");
}
