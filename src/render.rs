extern crate x11rb;
use crate::render::render_entities;

use core::f64;
use std::fmt::{self, Debug, Display, Write as _};
use std::sync::{self, mpsc};
use std::{thread, time};

use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::cookie::Cookie;
use x11rb::errors::{ConnectError, ConnectionError, ReplyError, ReplyOrIdError};
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::argparse::Args;
use crate::math_types::GameVector;
use crate::*;
use xkeysym::KeyCode;

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

pub struct ProtoKeysymmer<'a, D: Connection> {
    cookie: Cookie<'a, D, GetKeyboardMappingReply>,
    min_keycode: u8,
}

impl<'a, D: Connection> ProtoKeysymmer<'a, D> {
    pub fn init(display: &'a D) -> Result<Self, XError> {
        let setup = display.setup();
        Ok(Self {
            cookie: display.get_keyboard_mapping(
                setup.min_keycode,
                setup.max_keycode - setup.min_keycode + 1,
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
        xkeysym::keysym(
            KeyCode::new(code as u32),
            0,
            KeyCode::new(self.min_keycode as u32),
            self.keysyms.keysyms_per_keycode,
            &self.keysyms.keysyms,
        )
        .map(|sym| sym.into())
    }
}

fn load_the_font(args: &Args, display: &impl Connection, font_id: u32) -> Result<bool, XError> {
    const FONT: &[u8] = b"-misc-dogica pixel-medium-r-normal--0-0-0-0-p-0-iso8859-15";
    const FONT_PATH: &str = "fonts/Dogica";

    // Check to see if our desired font is already findable. If so, load it immediately and
    // return.
    let font_list = display.list_fonts(u16::MAX, FONT)?.reply()?;
    if font_list
        .names
        .iter()
        .map(|entry| &entry.name)
        .any(|entry| entry == FONT)
    {
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
        }
    };
    my_fonts.push(FONT_PATH);

    match my_fonts.try_exists() {
        Ok(false) => {
            let error = errors::ResourceIntegrityError::associate_path(
                &args.assets_dir,
                errors::ResourceError::from(my_fonts.as_path()),
            );
            warn!(error);
            display.open_font(font_id, b"fixed")?;
            return Ok(false);
        }
        Err(_) => {
            let error = errors::ResourceIntegrityError::associate_path(
                &args.assets_dir,
                errors::ResourceError::from(my_fonts.as_path()),
            );
            warn!(error, "failed to check if font directory exists");
            display.open_font(font_id, b"fixed")?;
            return Ok(false);
        }
        _ => (),
    }

    // Await the X font path. Does it contain the dogica font directory?
    // If yes, then that's a problem, because it previously failed with that directory in-place to
    // the best of our knowledge. Load fixed and log a reasonable error.
    // TODO: Add additional check for this path but with a trailing forward slash.
    let old_font_path = old_font_path.reply()?;
    if old_font_path
        .path
        .iter()
        .map(|entry| &entry.name)
        .any(|entry| entry == my_fonts.as_os_str().as_bytes())
    {
        let error = errors::ResourceIntegrityError::associate_path(
            &args.assets_dir,
            errors::ResourceError::from(my_fonts.as_path()),
        );
        fatal!(error);
        display.open_font(font_id, b"fixed")?;
        return Ok(false);
    }

    // Set the font path to include the Dogica font directory, now that it looks reasonable that
    // the reason it initially failed is due to having the font path set incorrectly.
    let my_fonts_as_str = Str {
        name: Vec::from(my_fonts.as_os_str().as_bytes()),
    };
    let mut new_font_path = old_font_path;
    new_font_path.path.insert(0, my_fonts_as_str);
    let old_font_path = &new_font_path.path[1..];
    display.set_font_path(&new_font_path.path)?;

    // Check the font we initially checked for once again. Load it if successfully found.
    let font_list = display.list_fonts(u16::MAX, FONT)?.reply()?;
    if font_list
        .names
        .iter()
        .map(|entry| &entry.name)
        .any(|entry| entry == FONT)
    {
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
    pub display: RustConnection,
    pipeline: &'a sync::Mutex<render_pipeline::ReadRenderInfo<A, P>>,
    pub gc: u32,
    pub player_gc: u32,
    pub player_pixel_normal: u32,
    pub player_pixel_boost: u32,
    pub player_pixel_big_boost: u32,
    pub font: u32,
    pub window: u32,
    pub wm_delete_window: u32,
}

const WIDTH: u16 = 800;
const HEIGHT: u16 = 600;
const LINE_THICKNESS: u32 = 5;
const MID_X: i32 = WIDTH as i32 / 2;
const MID_Y: i32 = HEIGHT as i32 / 2;
const SCALE_FACTOR: i64 = 240;

impl<'a, const A: usize, const P: usize> DisplayState<'a, A, P> {
    pub fn new(
        args: &Args,
        pipeline: &'a sync::Mutex<render_pipeline::ReadRenderInfo<A, P>>,
    ) -> Result<(Self, Keysymmer), XError> {
        let (display, screen) = x11rb::connect(None)?;
        let screen = &display.setup().roots[screen];

        let player_pixel_normal =
            display.alloc_color(screen.default_colormap, 0, u16::MAX / 2, u16::MAX)?;
        let player_pixel_boost =
            display.alloc_color(screen.default_colormap, u16::MAX, u16::MAX / 2, 0)?;
        let player_pixel_big_boost =
            display.alloc_color(screen.default_colormap, u16::MAX, 0, 0)?;

        // create an appropriate GC and window
        // generate XIDs for window and GC
        let (window, gc, player_gc, font) = (
            display.generate_id()?,
            display.generate_id()?,
            display.generate_id()?,
            display.generate_id()?,
        );

        let keysymmer = ProtoKeysymmer::init(&display)?;

        let delete_signal = display.intern_atom(false, b"WM_DELETE_WINDOW")?;
        let wm_protocols = display.intern_atom(false, b"WM_PROTOCOLS")?;

        display.create_window(
            COPY_DEPTH_FROM_PARENT,
            window,
            screen.root,
            0,
            0,
            WIDTH,
            HEIGHT,
            0,
            WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &CreateWindowAux::default()
                .background_pixel(screen.black_pixel)
                .event_mask(
                    EventMask::EXPOSURE
                        | EventMask::KEY_PRESS
                        | EventMask::KEY_RELEASE
                        | EventMask::FOCUS_CHANGE,
                ),
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
        display.change_property32(
            PropMode::REPLACE,
            window,
            wm_protocols,
            AtomEnum::ATOM,
            &[delete_signal],
        )?;

        // Set the window title and class
        display.change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            b"xpedition-game",
        )?;
        display.change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            b"Xpedition Prototype",
        )?;

        // Actually get the pixel value for the main player color
        let player_pixel_normal = player_pixel_normal.reply()?.pixel;

        // create GCs
        display.create_gc(
            gc,
            screen.root,
            &CreateGCAux::default()
                .foreground(screen.white_pixel)
                .line_width(LINE_THICKNESS)
                .font(font),
        )?;
        display.create_gc(
            player_gc,
            screen.root,
            &CreateGCAux::default()
                .foreground(player_pixel_normal)
                .line_width(LINE_THICKNESS),
        )?;

        // Await everything else we haven't used yet
        let player_pixel_boost = player_pixel_boost.reply()?.pixel;
        let player_pixel_big_boost = player_pixel_big_boost.reply()?.pixel;
        let keysymmer = keysymmer.finish_init()?;

        display.flush()?;

        Ok((
            Self {
                display,
                pipeline,
                gc,
                player_gc,
                player_pixel_normal,
                player_pixel_boost,
                player_pixel_big_boost,
                font,
                window,
                wm_delete_window: delete_signal,
            },
            keysymmer,
        ))
    }

    pub fn gimme_display(&self) -> &RustConnection {
        &self.display
    }

    pub fn gimme_wm_delete_window(&self) -> u32 {
        self.wm_delete_window
    }

    pub fn run(&self, render_events: mpsc::Receiver<events::RenderEvent>) -> Result<(), XError> {
        // thread::sleep(time::Duration::from_secs(1)); // TODO: Remove this and fix panic.
        let mut debug_text_buffer = String::new();

        'biggie: loop {
            // Handle events from the master event handler.
            for event in render_events.try_iter() {
                use events::RenderEvent as re;
                match event {
                    re::CloseWindow => break 'biggie,
                }
            }

            self.display.clear_area(true, self.window, 0, 0, 0, 0)?;

            let pipeline_copy = {
                let handle = self
                    .pipeline
                    .lock()
                    .expect("FUCK FUCKING SHIT MY PIPE IS BROKEN AAA");
                handle.clone()
            };

            let player = match pipeline_copy.get_player() {
                Some(player) => player,
                None => continue,
            };

            debug_text_buffer.clear();
            let _ = write!(debug_text_buffer, "new: {:?}", player.end_position);
            self.display
                .image_text8(self.window, self.gc, 10, 20, debug_text_buffer.as_bytes())?;

            self.render_player(&player)?;

            self.display.flush()?;
            thread::sleep(time::Duration::from_millis(1));
        }

        Ok(())
    }

    // Should I have this rendering function do lerping of the player position on its own, or have
    // it fed in an already lerped player info? I think the former is cleaner and will be more
    // consistent in code.
    pub fn render_player(&self, player: &render_pipeline::PlayerRender) -> Result<(), XError> {
        // TODO: Make a Radians type that is more restricted in how it's used to make calculations
        // more robust.
        const fn degrees_to_radians(degrees: f64) -> f64 {
            degrees / 180.0 * f64::consts::PI
        }

        struct LineAndAngle {
            length: i64,
            angle: f64,
        }

        const POINTS: [LineAndAngle; 5] = [
            LineAndAngle {
                length: 10,
                angle: 0.0,
            },
            LineAndAngle {
                length: 10,
                angle: degrees_to_radians(130.0),
            },
            LineAndAngle {
                length: 1,
                angle: degrees_to_radians(180.0),
            },
            LineAndAngle {
                length: 10,
                angle: degrees_to_radians(230.0),
            },
            LineAndAngle {
                length: 10,
                angle: 0.0,
            },
        ];

        let transformed_points = POINTS.map(|input| Point {
            x: ((f64::sin(input.angle + player.end_rotation as f64) * (input.length as f64)) as i64
                + (WIDTH as i64 / 2)
                + (player.end_position.x / SCALE_FACTOR)) as i16,
            y: ((f64::cos(input.angle + player.end_rotation as f64) * (input.length as f64)) as i64
                * -1
                + (HEIGHT as i64 / 2)
                + (player.end_position.y / SCALE_FACTOR)) as i16,
        });

        self.display.poly_line(
            CoordMode::ORIGIN,
            self.window,
            self.player_gc,
            &transformed_points,
        )?;
        Ok(())
    }
}
