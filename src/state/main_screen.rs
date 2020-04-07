use amethyst::{
    input::{get_key, is_close_requested, is_key_down, ElementState, VirtualKeyCode},
    prelude::*,
};

use super::PlayingMap;

#[derive(Default)]
pub struct MainScreen {
    started: bool,
}

impl SimpleState for MainScreen {
    // until we have a functional full-screen menu, just jump into the game
    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if !self.started {
            self.started = true;
            Trans::Push(Box::new(PlayingMap::default()))
        } else {
            Trans::Quit
        }
    }

    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            // Check if the window should be closed
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }

            // Listen to any key events
            if let Some(event) = get_key(&event) {
                match event {
                    (_, ElementState::Pressed) => {
                        return Trans::Push(Box::new(PlayingMap::default()))
                    }
                    _ => {}
                }
            }

            // If you're looking for a more sophisticated event handling solution,
            // including key bindings and gamepad support, please have a look at
            // https://book.amethyst.rs/stable/pong-tutorial/pong-tutorial-03.html#capturing-user-input
        }

        // Keep going
        Trans::None
    }
}
