extern crate x11rb;

use std::ops::Deref;
use std::{time, thread, process};
use std::fmt::{self, Display, Debug, Write as _};

use x11rb::connection::Connection;
use x11rb::cookie::Cookie;
use x11rb::errors::{ConnectionError, ReplyOrIdError, ReplyError};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::{COPY_DEPTH_FROM_PARENT, rust_connection};

use xkeysym::KeyCode;
use xpedition::game_vector::GameVector;

use argh::{FromArgs, FromArgValue};
use std::path::{Path, PathBuf};

use notify_rust::{Notification, Urgency};
use std::io::Write as _;

// program idea: draw a path controlled by WASD, and have it never be cleared, and have the path
// color change over time.

// This type conceptually might be dumb.
#[derive(Debug)]
pub enum XError {
    Connection(ConnectionError),
    Reply(ReplyError),
    ReplyOrId(ReplyOrIdError),
}

impl Display for XError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(error) => write!(formatter, "Connection error: {}", error)?,
            Self::ReplyOrId(error) => write!(formatter, "Reply or ID error: {}", error)?,
            Self::Reply(error) => write!(formatter, "Reply error: {}", error)?,
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

impl From<ReplyError> for XError {
    fn from(value: ReplyError) -> Self {
        Self::Reply(value)
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
        })
    }

    pub fn finish_init(self) -> Result<Keysymmer, XError> {
        let keysyms = self.cookie.reply()?;
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
    pub fn code_to_sym(&self, code: Keycode, _page: u8) -> Option<xkeysym::Keysym> {
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

fn load_the_font(args: &Args, display: &impl Connection, font_id: u32) -> Result<bool, XError> {
    const FONT: &[u8] = b"-misc-dogica pixel-medium-r-normal--0-0-0-0-p-0-iso8859-15";
    const FONT_PATH: &str = "assets/fonts/Dogica";

    // We need a function to open the path for this font if it's not already in the font path.
    // That's something that:
    // * tries to see if this font exists
    // * if not, check the font path
    // * if the font path already contains the folder for the needed font, use fixed
    // * else, set the font path to also include the dogma font
    // * Try checking again that the font exists. If not, use fixed
    // * else, use the dogma font we wanted to use this whole time.
    // * If the old font path previously didn't have the dogma font path, restore the original font
    // path after opening the font.
    
    let mut font_list = display.list_fonts(u16::MAX, FONT)?.reply()?;
    if font_list.names.iter().map(|entry| &entry.name).any(|entry| entry == FONT) {
        display.open_font(font_id, FONT)?;
        return Ok(true);
    }

    // if not, check the font path. Does it have our custom font path already?
    let old_font_path = display.get_font_path()?;
    // While waiting, let's get the absolute path of the font directory.
    // ...how We first need to be able to find the assets folder. That information should get
    // passed on the command line as a flag, but if that flag is absent, how about we default to
    // {program_path}/assets?

    Ok(false)
}

fn while_connected(args: &Args, display: &impl Connection, screen: &Screen) -> Result<(), XError> {
    const WIDTH: u16 = 800;
    const HEIGHT: u16 = 600;
    const LINE_THICKNESS: u32 = 5;
    const MID_X: i32 = WIDTH as i32 / 2;
    const MID_Y: i32 = HEIGHT as i32 / 2;
    const SCALE_FACTOR: i32 = 8;


    // create an appropriate GC and window
    // generate XIDs for window and GC
    let (window, gc, font) = (
        display.generate_id()?, display.generate_id()?, display.generate_id()?,
    );

    let keysymmer = ProtoKeysymmer::init(display)?;

    let delete_signal = display.intern_atom(false, b"WM_DELETE_WINDOW")?;
    let wm_protocols = display.intern_atom(false, b"WM_PROTOCOLS")?;


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
            .background_pixel(screen.black_pixel)
            .event_mask(EventMask::EXPOSURE | EventMask::KEY_PRESS | EventMask::KEY_RELEASE)
    )?;

    display.open_font(font, b"fixed")?;

    // map the window
    display.map_window(window)?;

    // await the WM_DELETE_WINDOW atom and add it to WM_PROTOCOLS on our window
    let delete_signal = delete_signal.reply()?.atom;
    let wm_protocols = wm_protocols.reply()?.atom;
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

    // create GC
    display.create_gc (
        gc,
        screen.root,
        &CreateGCAux::default()
            .foreground(screen.white_pixel)
            .line_width(LINE_THICKNESS)
            .font(font)
    )?;

    let keysymmer = keysymmer.finish_init()?;
    display.flush()?;

    // instantiating player state
    let mut head_position = GameVector::<i32>::new(MID_X * SCALE_FACTOR, MID_Y * SCALE_FACTOR);
    let mut old_head_position = GameVector::<i32>::new(MID_X * SCALE_FACTOR, MID_Y * SCALE_FACTOR);
    let mut head_velocity = GameVector::<i32>::new(0, 0);
    let mut key_state = KeyState::new();

    let mut debug_text_buffer = String::new();

    // in a loop, handle keyboard inputs, Expose, and WM_DELETE_WINDOW
    // while also just drawing like normal.
    #[allow(unreachable_code)]
    Ok(loop {
        while let Some(event) = display.poll_for_event()? {
            match event {
                Event::Expose(_) => {
                    display.poly_line (
                        CoordMode::ORIGIN,
                        window,
                        gc,
                        &[(old_head_position / SCALE_FACTOR).into(), (head_position / SCALE_FACTOR).into()]
                    )?;
                },
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

        display.clear_area(true, window, 0, 0, 0, 0)?;

        debug_text_buffer.clear();
        let _ = write!(debug_text_buffer, "pos: {:?}", head_position);
        display.image_text8(window, gc, 10, 20, debug_text_buffer.as_bytes())?;
        debug_text_buffer.clear();
        let _ = write!(debug_text_buffer, "vel: {:?}", head_velocity);
        display.image_text8(window, gc, 10, 40, debug_text_buffer.as_bytes())?;

        display.poly_line (
            CoordMode::ORIGIN,
            window,
            gc,
            &[(old_head_position / SCALE_FACTOR).into(), (head_position / SCALE_FACTOR).into()]
        )?;

        head_velocity += key_state.make_vector();
        old_head_position = head_position;
        head_position += head_velocity;

        display.flush()?;
        thread::sleep(time::Duration::from_millis(16));
    })
}

fn oopsie(error: XError) -> ! {
    // Possibly the most "trust me bro" string handling code I've written in this langauge
    let mut buffer = Vec::<u8>::with_capacity(256);
    let _ = write!(buffer, "{error}");
    let first_length = buffer.len();
    for i in 0..first_length {
        let replacement: &'static [u8] = match unsafe {buffer.get_unchecked(i)} {
            b'<' => b"&lt;",
            b'>' => b"&gt;",
            b'&' => b"&amp;",
            b'"' => b"&quot;",
            something_else => {
                buffer.push(*something_else);
                continue;
            },
        };

        buffer.extend(replacement);
    }
    let sanitized = unsafe {str::from_utf8_unchecked(&buffer[first_length..])};

    Notification::new()
        .summary("Xpedition has crashed! :(")
        .body(sanitized)
        .timeout(time::Duration::from_secs(10))
        .show().unwrap();

    process::exit(67)
}

#[derive(Debug)]
struct AssetPath(PathBuf);
impl Deref for AssetPath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromArgValue for AssetPath {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        let mut returned = PathBuf::new();

        let mut current = value;
        while let Some((left, right)) = current.split_once("{") {
            returned.push(left);

            // get the closing brace
            if let Some((inside, outside)) = right.split_once("}") {
                match inside {
                    "brace" => returned.push("{"),
                    "self" => returned.push("urmomlol"),
                    nono => return Err(format!("Invalid substitution inside brace: {nono}")),
                }

                current = outside;
            } else {
                return Err("Forgot closing brace".to_string());
            }
        }

        returned.push(current);
        Ok(Self(returned))
    }
}

#[derive(FromArgs, Debug)]
/// Silly little game made with X11 APIs.
struct Args {
    /// specify assets directory
    /// (default is {self}/assets, {self} substitutes for the program's running directory)
    #[argh(option, short = 'a', default = "AssetPath(PathBuf::from(\"{self}/assets\"))")]
    assets_dir: AssetPath,
}

// TODO: Figure out the fonts situation
    // I want fonts to reasonably scale up as I use them, at least up to a 4K screen. This should
    // be possible with the X core font rendering, but I'll need to do it by having multiple fonts
// TODO: make better physics
// TODO: have the GC alternate color
// TODO: experiment with rotating graphics and images

fn main() {
    let args: Args = argh::from_env();
    println!("Args: {:?}", args);

    let (display, screen) = x11rb::connect(None).unwrap();
    let screen = &display.setup().roots[screen];
    if let Err(error) = while_connected(&args, &display, screen) {
        oopsie(error)
    }

    println!("Goodbye!!~ >w<");
}

