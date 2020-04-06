use amethyst::core::SystemDesc;
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Read, System, SystemData, World, Write};
use amethyst::input::VirtualKeyCode;
use amethyst::input::{InputHandler, StringBindings};
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
}

impl Button {
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
pub struct InputState {
    pub actions_pending: VecDeque<Action>,
    pub keys_down: HashSet<Button>,
    pub movements_down: VecDeque<Direction>,
}

impl InputState {
    pub fn pop_action(&mut self) -> Vec<Action> {
        // if let Some(action) =  {
        //     return vec![action];
        // }

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
}
#[derive(SystemDesc)]
pub struct InputSystem;

impl InputSystem {
    pub fn init(world: &mut World) {
        world.insert(InputState::default());
    }
}

impl<'s> System<'s> for InputSystem {
    type SystemData = (
        Write<'s, InputState>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (mut input_state, input): Self::SystemData) {
        for button in Button::all() {
            match (
                input_state.keys_down.contains(button),
                input.key_is_down(button.to_key()),
            ) {
                (true, true) => {}
                (false, false) => {}
                (true, false) => {
                    input_state.keys_down.remove(button);
                    if let Button::Move(dir) = button {
                        input_state.movements_down.retain(|d| d != dir);
                    }
                }
                (false, true) => {
                    input_state.keys_down.insert(*button);
                    if let Button::Move(dir) = button {
                        input_state.movements_down.push_front(*dir);
                        let fire = input.key_is_down(Button::Fire.to_key());
                        input_state.actions_pending.push_back(Action {
                            fire,
                            direction: *dir,
                        });
                    }
                }
            }
        }
    }
}
