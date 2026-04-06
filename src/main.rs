extern crate x11rb;

use std::{time, thread};
use std::fmt::{self, Display, Debug};

use x11rb::connection::Connection;
use x11rb::cookie::Cookie;
use x11rb::errors::{ConnectionError, ReplyError, ReplyOrIdError};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::COPY_DEPTH_FROM_PARENT;

use xkeysym::KeyCode;
use xpedition::game_vector::GameVector;
use xpedition::{physics, game_vector};

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

#[derive(Default)]
pub struct KeyState {
    w: bool,
    s: bool,
    a: bool,
    d: bool,
}

impl KeyState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, key: xkeysym::Keysym, pressed: bool) {
        use xkeysym::key as k;
        match key.raw() {
            k::W | k::w => self.w = pressed,
            k::S | k::s => self.s = pressed,
            k::A | k::a => self.a = pressed,
            k::D | k::d => self.d = pressed,
            _ => (),
        }
    }

    pub fn make_vector(&self) -> GameVector<i32> {
        let mut returned = GameVector::new(0, 0);
        if self.w {
            returned.y -= 1;
        }
        if self.s {
            returned.y += 1;
        }
        if self.a {
            returned.x -= 1;
        }
        if self.d {
            returned.x += 1;
        }

        returned
    }
}

pub struct ProtoKeysymmer<'a, D: Connection> {
    cookie: Cookie<'a, D, GetKeyboardMappingReply>,
    min_keycode: u8,
    max_keycode: u8,
}

impl<'a, D: Connection> ProtoKeysymmer<'a, D> {
    pub fn init(display: &'a D) -> Result<Self, XError> {
        let setup = display.setup();
        Ok(Self {
            cookie: display.get_keyboard_mapping (
                setup.min_keycode,
                setup.max_keycode - setup.min_keycode + 1
            )?,
            min_keycode: setup.min_keycode,
            max_keycode: setup.max_keycode,
        })
    }

    pub fn finish_init(self) -> Result<Keysymmer, XError> {
        let keysyms = self.cookie.reply().map_err(|e| ReplyOrIdError::from(e))?;
        Ok(Keysymmer {
            keysyms,
            min_keycode: self.min_keycode,
        })
    }
}

pub struct Keysymmer {
    keysyms: GetKeyboardMappingReply,
    min_keycode: u8,
}

impl Keysymmer {
    pub fn code_to_sym(&self, code: Keycode, page: u8) -> Option<xkeysym::Keysym> {
        // TODO: wtf do I do with the pages???
        xkeysym::keysym (
            KeyCode::new(code as u32),
            0,
            KeyCode::new(self.min_keycode as u32),
            self.keysyms.keysyms_per_keycode,
            &self.keysyms.keysyms
        ).map(|sym| sym.into())
    }
}

fn while_connected(display: &impl Connection, screen: &Screen) -> Result<(), XError> {
    const WIDTH: u16 = 800;
    const HEIGHT: u16 = 600;
    const LINE_THICKNESS: u32 = 4;
    const MID_X: i32 = WIDTH as i32 / 2;
    const MID_Y: i32 = HEIGHT as i32 / 2;

    // create an appropriate GC and window
    // generate XIDs for window and GC
    let (window, gc) = (
        display.generate_id().map_err(|e| XError::from(e))?,
        display.generate_id().map_err(|e| XError::from(e))?,
    );

    let keysymmer = ProtoKeysymmer::init(display)?;

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
            .event_mask(EventMask::EXPOSURE | EventMask::KEY_PRESS | EventMask::KEY_RELEASE)
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

    // Set the window title and class
    display.change_property8 (
        PropMode::REPLACE,
        window,
        AtomEnum::WM_CLASS,
        AtomEnum::STRING,
        b"xpedition-game",
    )?;
    display.change_property8 (
        PropMode::REPLACE,
        window,
        AtomEnum::WM_NAME,
        AtomEnum::STRING,
        b"Xpedition Prototype",
    )?;

    let keysymmer = keysymmer.finish_init()?;
    display.flush()?;

    // instantiating player state
    let mut head_position = GameVector::<i32>::new(MID_X, MID_Y);
    let mut old_head_position = GameVector::<i32>::new(MID_X, MID_Y);
    let mut head_velocity = GameVector::<i32>::new(0, 0);
    let mut key_state = KeyState::new();

    // in a loop, handle keyboard inputs, Expose, and WM_DELETE_WINDOW
    // while also just drawing like normal.
    #[allow(unreachable_code)]
    Ok(loop {
        while let Some(event) = display.poll_for_event()? {
            match event {
                Event::KeyPress(event) => {
                    let sym = keysymmer.code_to_sym(event.detail, 0).unwrap();
                    key_state.update(sym, true);
                },
                Event::KeyRelease(event) => {
                    let sym = keysymmer.code_to_sym(event.detail, 0).unwrap();
                    key_state.update(sym, false);
                },
                Event::ClientMessage(ClientMessageEvent {format: 32, data, ..}) => {
                    if data.as_data32()[0] == delete_signal {
                        return Ok(());
                    }
                },
                _ => (),
            }
        }

        display.poly_line (
            CoordMode::ORIGIN,
            window,
            gc,
            &[old_head_position.into(), head_position.into()]
        )?;

        head_velocity += key_state.make_vector();
        old_head_position = head_position;
        head_position += head_velocity;

        display.flush()?;
        thread::sleep(time::Duration::from_millis(32));
    })
}

fn main() {
    let (display, screen) = x11rb::connect(None).unwrap();
    let screen = &display.setup().roots[screen];
    while_connected(&display, screen).unwrap();

    // TODO: handle notification on crash here

    println!("Goodbye!!~ >w<");
}
