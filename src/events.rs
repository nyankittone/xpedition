// A seperate thread of execution shall listen to events from the X server, and then propogate those
// events to the threads that need to listen to them. i.e., the physics thread needs to listen to
// stuff that could update the player's motion, pause the game, or the application closing, and the
// renderer might need to listen to messages for opening menus, mouse clicks, window resizes, maybe
// expose events, and also closing the application.

use std::sync::mpsc;

use x11rb::connection::Connection;
use x11rb::rust_connection::RustConnection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;

use crate::render::{XError, Keysymmer};

pub struct PlayerControl {
    main_forward: bool,
    big_forward: bool,
    rotate_left: bool,
    rotate_right: bool,
}

pub enum PlayerSet {
    Reverse,
}

pub enum PhysicsEvent {
    PauseGame,
    ResumeGame,
    Stop,
    PlayerControl(PlayerControl),
    PlayerSet(PlayerSet),
}

pub enum RenderEvent {
    CloseWindow,
}

pub fn listen(
    send_physics: &mpsc::Sender<PhysicsEvent>,
    send_render: &mpsc::Sender<RenderEvent>,
    display: &impl Connection,
    keysymmer: &Keysymmer,
    wm_delete_window: Atom,
) -> Result<(), XError> {
    loop {
        match display.wait_for_event()? {
            Event::KeyPress(event) => {
                let sym = keysymmer.code_to_sym(event.detail, 0).unwrap();
                key_state.update(sym, true);
            },
            Event::KeyRelease(event) => {
                let sym = keysymmer.code_to_sym(event.detail, 0).unwrap();
                key_state.update(sym, false);
            },
            Event::ClientMessage(ClientMessageEvent {format: 32, data, ..}) => {
                if data.as_data32()[0] == wm_delete_window {
                    send_physics.send(PhysicsEvent::Stop).unwrap();
                    send_render.send(RenderEvent::CloseWindow).unwrap();
                    return Ok(());
                }
            },
            _ => (),
        }
    }
}
