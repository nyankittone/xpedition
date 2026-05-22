extern crate x11rb;
use crate::render::render_entities;

use std::{time, thread};
use std::fmt::{self, Display, Debug, Write as _};
use std::sync;

use x11rb::connection::Connection;
use x11rb::cookie::Cookie;
use x11rb::errors::{ConnectionError, ReplyOrIdError, ReplyError, ConnectError};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::COPY_DEPTH_FROM_PARENT;

use xkeysym::KeyCode;
use crate::game_vector::GameVector;
use crate::*;
use crate::argparse::Args;

use std::os::unix::ffi::OsStrExt;

#[derive(Debug)]
pub enum XError {
    ConnectionDied(ConnectionError),
    FailedToConnect(ConnectError),
    Reply(ReplyError),
    ReplyOrId(ReplyOrIdError),
}

impl Display for XError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionDied(error) => write!(formatter, "Connection error: {}", error)?,
            Self::FailedToConnect(error) => write!(formatter, "Failed to connect: {}", error)?,
            Self::ReplyOrId(error) => write!(formatter, "Reply or ID error: {}", error)?,
            Self::Reply(error) => write!(formatter, "Reply error: {}", error)?,
        }

        Ok(())
    }
}

impl std::error::Error for XError {}

impl From<ConnectionError> for XError {
    fn from(value: ConnectionError) -> Self {
        Self::ConnectionDied(value)
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

impl From<ConnectError> for XError {
    fn from(value: ConnectError) -> Self {
        Self::FailedToConnect(value)
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

pub struct DisplayState<'a, const A: usize, const P: usize> {
    display: RustConnection,
    pipeline: &'a sync::Mutex<render_pipeline::ReadRenderInfo<A, P>>,
    gc: u32,
    font: u32,
    window: u32,
    wm_delete_window: u32,
}

const WIDTH: u16 = 800;
const HEIGHT: u16 = 600;
const LINE_THICKNESS: u32 = 5;
const MID_X: i32 = WIDTH as i32 / 2;
const MID_Y: i32 = HEIGHT as i32 / 2;
const SCALE_FACTOR: i32 = 8;

impl<'a, const A: usize, const P: usize> DisplayState<'a, A, P> {
    pub fn new(args: &Args, pipeline: &'a sync::Mutex<render_pipeline::ReadRenderInfo<A, P>>) -> Result<(Self, Keysymmer), XError> {
        let (display, screen) = x11rb::connect(None)?;
        let screen = &display.setup().roots[screen];

        // create an appropriate GC and window
        // generate XIDs for window and GC
        let (window, gc, font) = (
            display.generate_id()?, display.generate_id()?, display.generate_id()?,
        );

        let keysymmer = ProtoKeysymmer::init(&display)?;

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
        if load_the_font(args, &display, font)? {
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

        Ok((Self {
            display,
            pipeline,
            gc,
            font,
            window,
            wm_delete_window: delete_signal,
        }, keysymmer))
    }

    pub fn gimme_display(&self) -> &RustConnection {
        &self.display
    }

    pub fn run(&mut self, keysymmer: &Keysymmer) -> Result<(), XError> {
        // instantiating player state
        let mut head_position = GameVector::<i32>::new(MID_X * SCALE_FACTOR, MID_Y * SCALE_FACTOR);
        let mut old_head_position = GameVector::<i32>::new(MID_X * SCALE_FACTOR, MID_Y * SCALE_FACTOR);
        let mut head_velocity = GameVector::<i32>::new(0, 0);
        let mut key_state = KeyState::new();

        let mut debug_text_buffer = String::new();

        loop {
            while let Some(event) = self.display.poll_for_event()? {
                match event {
                    Event::Expose(_) => {
                        self.display.poly_line (
                            CoordMode::ORIGIN,
                            self.window,
                            self.gc,
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
                        if data.as_data32()[0] == self.wm_delete_window {
                            return Ok(());
                        }
                    },
                    _ => (),
                }
            }

            self.display.clear_area(true, self.window, 0, 0, 0, 0)?;
            
            let pipeline_copy = {
                let handle = self.pipeline.lock().expect("FUCK FUCKING SHIT MY PIPE IS BROKEN AAA");
                handle.clone()
            };

            debug_text_buffer.clear();
            let _ = write!(debug_text_buffer, "old: {:?}", pipeline_copy.get_player().unwrap().start_position);
            self.display.image_text8(self.window, self.gc, 10, 20, debug_text_buffer.as_bytes())?;
            debug_text_buffer.clear();
            let _ = write!(debug_text_buffer, "new: {:?}", pipeline_copy.get_player().unwrap().end_position);
            self.display.image_text8(self.window, self.gc, 10, 40, debug_text_buffer.as_bytes())?;

            // self.display.poly_line (
            //     CoordMode::ORIGIN,
            //     self.window,
            //     self.gc,
            //     &[(old_head_position / SCALE_FACTOR).into(), (head_position / SCALE_FACTOR).into()]
            // )?;

            self.display.poly_fill_rectangle(self.window, self.gc, &[Rectangle {
                x: (pipeline_copy.get_player().unwrap().end_position.x / 1000000) as i16,
                y: (pipeline_copy.get_player().unwrap().end_position.y / 1000000) as i16,
                width: 20,
                height: 20,
            }]);

            head_velocity += key_state.make_vector();
            old_head_position = head_position;
            head_position += head_velocity;

            self.display.flush()?;
            thread::sleep(time::Duration::from_millis(16));
        }
    }

}
