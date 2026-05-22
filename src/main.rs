use std::io::Write as _;
use std::sync;
use std::thread;
use std::{process, time};

use notify_rust::Notification;
use x11rb::connection::Connection;
use xpedition::*;

fn oopsie(error: render::XError) -> ! {
    // Possibly the most "trust me bro" string handling code I've written in this langauge
    let mut buffer = Vec::<u8>::with_capacity(256);
    let _ = write!(buffer, "{error}");
    let first_length = buffer.len();
    for i in 0..first_length {
        let replacement: &'static [u8] = match unsafe { buffer.get_unchecked(i) } {
            b'<' => b"&lt;",
            b'>' => b"&gt;",
            b'&' => b"&amp;",
            b'"' => b"&quot;",
            something_else => {
                buffer.push(*something_else);
                continue;
            }
        };

        buffer.extend(replacement);
    }
    let sanitized = unsafe { str::from_utf8_unchecked(&buffer[first_length..]) };

    Notification::new()
        .summary("Xpedition has crashed! :(")
        .body(sanitized)
        .timeout(time::Duration::from_secs(10))
        .show()
        .unwrap();

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

pub fn start_game(args: &argparse::Args) -> Result<(), render::XError> {
    let mut render_pipeline = sync::Mutex::new(render_pipeline::ReadRenderInfo::<128, 512>::new());

    let mut error_from_scope: Option<render::XError> = None;
    thread::scope(|scope| {
        let handle = scope.spawn(|| tick::start_physics(&render_pipeline));

        let (mut renderer, keysymmer) = match render::DisplayState::new(args) {
            Ok(stuff) => stuff,
            Err(error) => {
                error_from_scope = Some(error);
                return;
            }
        };

        if let Err(error) = renderer.run(&keysymmer) {
            error_from_scope = Some(error);
            return;
        }

        handle.join().unwrap();
    });

    error_from_scope.map_or_else(|| Ok(()), |error| Err(error))
}

fn main() {
    let args: argparse::ProtoArgs = argh::from_env();
    let args = argparse::Args::new(args).unwrap();

    // Check that the passed in directory actually exists, as well as needed subdirectories
    match args.assets_dir.try_exists() {
        Ok(false) => {
            fatal!(&errors::ResourceError::from(args.assets_dir.as_path()));
            process::exit(67);
        }
        Ok(true) => (),
        Err(error) => {
            fatal!(&error, "cannot determine if asset directory exists");
            process::exit(67);
        }
    }

    println!("Args: {:?}", args);
    start_game(&args).unwrap_or_else(|error| oopsie(error));
    println!("Goodbye!!~ >w<");
}
