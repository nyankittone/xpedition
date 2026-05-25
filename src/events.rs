// A seperate thread of execution shall listen to events from the X server, and then propogate those
// events to the threads that need to listen to them. i.e., the physics thread needs to listen to
// stuff that could update the player's motion, pause the game, or the application closing, and the
// renderer might need to listen to messages for opening menus, mouse clicks, window resizes, maybe
// expose events, and also closing the application.

use std::sync::mpsc;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;
use xkeysym::{Keysym, key};

use crate::render::{Keysymmer, XError};

#[derive(Copy, Clone, Debug)]
pub struct PlayerControl {
    main_forward: bool,
    big_forward: bool,
    rotate_left: bool,
    rotate_right: bool,
}

impl PlayerControl {
    pub fn new() -> Self {
        Self {
            main_forward: false,
            big_forward: false,
            rotate_left: false,
            rotate_right: false,
        }
    }
}

impl Default for PlayerControl {
    fn default() -> Self {
        Self::new()
    }
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

fn update_player_control(control: &mut PlayerControl, sym: Keysym, with: bool) -> bool {
    // TODO: Implement a keymapping system that is used by this function in the future. For now,
    // directly reading the keysym is fine.
    let write_to: &mut bool = match sym.raw() {
        key::w | key::W => &mut control.main_forward,
        key::space => &mut control.big_forward,
        key::a | key::A => &mut control.rotate_left,
        key::d | key::D => &mut control.rotate_right,
        _ => return false,
    };

    eprintln!("I wanna set with {}, current val is {}", with, *write_to);

    if *write_to == with {
        eprintln!("Deas this EVER print?");
        return false;
    }

    *write_to = with;
    true
}

pub fn listen(
    send_physics: &mpsc::Sender<PhysicsEvent>,
    send_render: &mpsc::Sender<RenderEvent>,
    display: &impl Connection,
    keysymmer: &Keysymmer,
    wm_delete_window: Atom,
) -> Result<(), XError> {
    let mut player_control = PlayerControl::new();

    loop {
        match display.wait_for_event()? {
            Event::KeyPress(event) => {
                eprintln!("Got a KeyPress.");
                let sym = keysymmer.code_to_sym(event.detail, 0).unwrap();
                if update_player_control(&mut player_control, sym, true) {
                    eprintln!("Meow!\n");
                    send_physics
                        .send(PhysicsEvent::PlayerControl(player_control))
                        .unwrap();
                } else if {
                    let sym = sym.raw();
                    sym == key::s || sym == key::S
                } {
                    send_physics
                        .send(PhysicsEvent::PlayerSet(PlayerSet::Reverse))
                        .unwrap();
                }
            }
            Event::KeyRelease(event) => {
                eprintln!("Got a KeyRelease.");
                let sym = keysymmer.code_to_sym(event.detail, 0).unwrap();
                if update_player_control(&mut player_control, sym, false) {
                    send_physics
                        .send(PhysicsEvent::PlayerControl(player_control))
                        .unwrap();
                }
            }
            Event::ClientMessage(ClientMessageEvent {
                format: 32, data, ..
            }) => {
                if data.as_data32()[0] == wm_delete_window {
                    send_physics.send(PhysicsEvent::Stop).unwrap();
                    send_render.send(RenderEvent::CloseWindow).unwrap();
                    break;
                }
            }
            _ => (),
        }
    }

    Ok(())
}
