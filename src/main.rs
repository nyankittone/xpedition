extern crate x11rb;

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
use xpedition::argparse::{self, Args};
use xpedition::*;

use std::os::unix::ffi::OsStrExt;

use notify_rust::{Notification, Urgency};
use std::io::Write as _;

// program idea: draw a path controlled by WASD, and have it never be cleared, and have the path
// color change over time.

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
    const FONT_PATH: &str = "fonts/Dogica";

    // Check to see if our desired font is already findable. If so, load it immediately and
    // return.
    let font_list = display.list_fonts(u16::MAX, FONT)?.reply()?;
    if font_list.names.iter().map(|entry| &entry.name).any(|entry| entry == FONT) {
        display.open_font(font_id, FONT)?;
        return Ok(true);
    }

    // if not, check the font path. Does it have our custom font path already?
    let old_font_path = display.get_font_path()?;
    // While waiting, let's get the absolute path of the font directory.
    
    // TODO: Making this is cringe. Can we instead re-use some other path allocation later on?
    // Cool.
    let mut my_fonts = match std::path::absolute(args.assets_dir.as_path()) {
        Ok(path) => path,
        Err(error) => {
            warn!(error, "error trying to get font path");
            display.open_font(font_id, b"fixed")?;
            return Ok(false);
        },
    };
    my_fonts.push(FONT_PATH);
    
    match my_fonts.try_exists() {
        Ok(false) => {
            let error = errors::ResourceIntegrityError::associate_path (
                &args.assets_dir,
                errors::ResourceError::from(my_fonts.as_path())
            );
            warn!(error);
            display.open_font(font_id, b"fixed")?;
            return Ok(false);
        },
        Err(_) => {
            let error = errors::ResourceIntegrityError::associate_path (
                &args.assets_dir,
                errors::ResourceError::from(my_fonts.as_path())
            );
            warn!(error, "failed to check if font dirctory exists");
            display.open_font(font_id, b"fixed")?;
            return Ok(false);
        },
        _ => (),
    }

    // Await the X font path. Does it contain the dogica font directory?
    // If yes, then that's a problem, because it previously failed with that directory in-place to
    // the best of our knowledge. Load fixed and log a reasonable error.
    // TODO: Add additional check for this path but with a trailing forward slash.
    let old_font_path = old_font_path.reply()?;
    if old_font_path.path.iter().map(|entry| &entry.name).any(|entry| entry == my_fonts.as_os_str().as_bytes()) {
        let error = errors::ResourceIntegrityError::associate_path (
            &args.assets_dir,
            errors::ResourceError::from(my_fonts.as_path()),
        );
        fatal!(error);
        display.open_font(font_id, b"fixed")?;
        return Ok(false);
    }

    // Set the font path to include the Dogica font directory, now that it looks reasonable that
    // the reason it initially failed is due to having the font path set incorrectly.
    let my_fonts_as_str = Str { name: Vec::from(my_fonts.as_os_str().as_bytes()) };
    let mut new_font_path = old_font_path;
    new_font_path.path.insert(0, my_fonts_as_str);
    let old_font_path = &new_font_path.path[1..];
    display.set_font_path(&new_font_path.path)?;

    // Check the font we initially checked for once again. Load it if successfully found.
    let font_list = display.list_fonts(u16::MAX, FONT)?.reply()?;
    if font_list.names.iter().map(|entry| &entry.name).any(|entry| entry == FONT) {
        display.open_font(font_id, FONT)?;
        display.set_font_path(old_font_path)?;
        return Ok(true);
    }

    // Otherwise, just give up.
    display.open_font(font_id, b"fixed")?;
    display.set_font_path(old_font_path)?;
    println!("MEOW");
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
            .background_pixel(screen.black_pixel)
            .event_mask(EventMask::EXPOSURE | EventMask::KEY_PRESS | EventMask::KEY_RELEASE)
    )?;

    // Try to load the font lol
    // display.open_font(font, b"fixed")?;
    if load_the_font(args, display, font)? {
        info!("Successfully loaded the desired font! :3");
    } else {
        warn!("Falling back to using 'fixed'.");
    }

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

// TODO: Figure out the fonts situation
    // I want fonts to reasonably scale up as I use them, at least up to a 4K screen. This should
    // be possible with the X core font rendering, but I'll need to do it by having multiple fonts
    // that are bitmap at multiple different scales that the program can switch between
    // dynamically.

// TODO: Separate ticking logic, event handling, and rendering logic into seperate threads.
//   For making the ticker talk to the renderer, using a shared memory location behind a mutex is
//   probably how I'll do it. The shared memory location will contain instructions on how to update
//   the sprites on screen, adding new sprites to render, removing them, and how to interpolate them for
//   frames between ticks.
//   The event handler can use channels to send appropriate events to the
//   ticker and the renderer.
// TODO: Add a placeholder rendering thing for the protaganist
// TODO: Adjust physics so that we get natural decay in velcoity over time. (We may want to adjust
// this over time)

fn main() {
    let args: argparse::ProtoArgs = argh::from_env();
    let args = argparse::Args::new(args).unwrap();
    
    // Check that the passed in directory actually exists, as well as needed subdirectories
    match args.assets_dir.try_exists() {
        Ok(false) => {
            fatal!(&errors::ResourceError::from(args.assets_dir.as_path()));
            process::exit(67);
        },
        Ok(true) => (),
        Err(error) => {
            fatal!(&error, "cannot determine if asset directory exists");
            process::exit(67);
        },
    }

    println!("Args: {:?}", args);

    let (display, screen) = x11rb::connect(None).unwrap();
    let screen = &display.setup().roots[screen];
    if let Err(error) = while_connected(&args, &display, screen) {
        oopsie(error)
    }

    println!("Goodbye!!~ >w<");
}

