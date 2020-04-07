use amethyst::{
    assets::Handle,
    ecs::prelude::*,
    input::{get_key, is_close_requested, is_key_down, ElementState, VirtualKeyCode},
    prelude::*,
    renderer::SpriteSheet,
    window::ScreenDimensions,
};

use crate::{camera, grid, input};
use log::info;

#[derive(Default)]
pub struct PlayingMap<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
    sprites: Option<Handle<SpriteSheet>>,
    grid: grid::GridState,
    tick_count: u64,
}

impl<'a, 'b> SimpleState for PlayingMap<'a, 'b> {
    // On start will run when this state is initialized. For more
    // state lifecycle hooks, see:
    // https://book.amethyst.rs/stable/concepts/state.html#life-cycle
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Get the screen dimensions so we can initialize the camera and
        // place our sprites correctly later. We'll clone this since we'll
        // pass the world mutably to the following functions.
        let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

        // Load our sprites and display them
        let sprites = super::load_sprites(world);
        self.sprites = Some(sprites.clone());
        self.grid.init(world, sprites);
        input::InputSystem::init(world);
        camera::CameraSystem::init(world, &dimensions);

        // Create the `DispatcherBuilder` and register some `System`s that should only run for this `State`.
        let mut dispatcher_builder = DispatcherBuilder::new();

        dispatcher_builder.add(input::InputSystem, "own_input_system", &[]);
        dispatcher_builder.add(camera::CameraSystem::default(), "camera_system", &[]);
        // Build and setup the `Dispatcher`.
        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        self.grid.deinit(world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world);
        }

        camera::CameraSystem::update_screen_dimensions(&mut data.world);
        SimpleTrans::None
    }

    fn fixed_update(&mut self, mut data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        self.tick_count += 1;
        if self.tick_count % 8 == 0 {
            self.grid.run_tick(&mut data.world);
        }

        SimpleTrans::None
    }

    fn handle_event(
        &mut self,
        mut data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            // Check if the window should be closed
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Pop;
            }

            // Listen to any key events
            if let Some(event) = get_key(&event) {
                info!("handling key event: {:?}", event);
                match event {
                    (VirtualKeyCode::R, ElementState::Pressed) => {
                        self.restart_map(&mut data.world);
                    }
                    (VirtualKeyCode::Add, ElementState::Pressed)
                    | (VirtualKeyCode::Equals, ElementState::Pressed) => {
                        let zoom = data.world.read_resource::<camera::ZoomLevel>().clone().up();
                        data.world.insert(zoom);
                    }
                    (VirtualKeyCode::Subtract, ElementState::Pressed) => {
                        let zoom = data
                            .world
                            .read_resource::<camera::ZoomLevel>()
                            .clone()
                            .down();
                        data.world.insert(zoom);
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

impl<'a, 'b> PlayingMap<'a, 'b> {
    fn restart_map(&mut self, world: &mut World) {
        self.grid.deinit(world);
        self.grid
            .init(world, self.sprites.clone().expect("sprites loaded"));
    }
}
