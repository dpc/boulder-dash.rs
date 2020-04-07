use amethyst::input::{ElementState, VirtualKeyCode};
use std::collections::{HashSet, VecDeque};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Button {
    Move(Direction),
    Fire,
}

impl Button {
    fn all() -> &'static [Self] {
        use Button::*;
        use Direction::*;
        // Important: fire is first, so it's processed first
        &[Fire, Move(Up), Move(Down), Move(Left), Move(Right)]
    }

    fn from_key(key: VirtualKeyCode) -> Option<Self> {
        use Button::*;
        use Direction::*;
        Some(match key {
            VirtualKeyCode::Left => Move(Left),
            VirtualKeyCode::Right => Move(Right),
            VirtualKeyCode::Up => Move(Up),
            VirtualKeyCode::Down => Move(Down),
            VirtualKeyCode::RControl => Fire,
            _ => return None,
        })
    }

    fn to_key(self) -> VirtualKeyCode {
        use Button::*;
        use Direction::*;
        match self {
            Move(Up) => VirtualKeyCode::Up,
            Move(Down) => VirtualKeyCode::Down,
            Move(Left) => VirtualKeyCode::Left,
            Move(Right) => VirtualKeyCode::Right,
            Fire => VirtualKeyCode::LControl,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Action {
    pub fire: bool,
    pub direction: Direction,
}

#[derive(Debug, Default)]
pub struct InputTracker {
    pub actions_pending: VecDeque<Action>,
    pub keys_down: HashSet<Button>,
    pub movements_down: VecDeque<Direction>,
}

impl InputTracker {
    pub fn pop_action(&mut self) -> Vec<Action> {
        self.actions_pending
            .pop_front()
            .into_iter()
            .chain(
                self.movements_down
                    .iter()
                    .take(2)
                    .copied()
                    .map(|direction| Action {
                        direction,
                        fire: self.keys_down.contains(&Button::Fire),
                    }),
            )
            .collect()
    }

    pub fn handle_key(&mut self, (key, key_state): (VirtualKeyCode, ElementState)) {
        let button = if let Some(button) = Button::from_key(key) {
            button
        } else {
            return;
        };

        match (
            self.keys_down.contains(&button),
            key_state == ElementState::Pressed,
        ) {
            (true, true) => {}
            (false, false) => {}
            (true, false) => {
                self.keys_down.remove(&button);
                if let Button::Move(dir) = button {
                    self.movements_down.retain(|d| d != &dir);
                }
            }
            (false, true) => {
                self.keys_down.insert(button);
                if let Button::Move(dir) = button {
                    self.movements_down.push_front(dir);
                    let fire = self.keys_down.contains(&Button::Fire);
                    self.actions_pending.push_back(Action {
                        fire,
                        direction: dir,
                    });
                }
            }
        }
    }
}
