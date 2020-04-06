use amethyst::{
    assets::Handle,
    assets::{AssetStorage, Loader},
    ecs::prelude::*,
    input::{get_key, is_close_requested, is_key_down, ElementState, VirtualKeyCode},
    prelude::*,
    renderer::{ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
    window::ScreenDimensions,
};

use crate::{camera, grid, input};
use log::info;

pub struct MainScreen;

impl SimpleState for MainScreen {
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
                info!("handling key event: {:?}", event);
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

#[derive(Default)]
pub struct PlayingMap<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
    sprites: Option<Handle<SpriteSheet>>,
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
        let sprites = load_sprites(world);
        self.sprites = Some(sprites.clone());
        grid::GridRulesSystem::init(world, sprites);
        input::InputSystem::init(world);
        camera::CameraSystem::init(world, &dimensions);

        // Create the `DispatcherBuilder` and register some `System`s that should only run for this `State`.
        let mut dispatcher_builder = DispatcherBuilder::new();

        dispatcher_builder.add(input::InputSystem, "own_input_system", &[]);
        dispatcher_builder.add(
            grid::GridRulesSystem,
            "grid_object_system",
            &["own_input_system"],
        );
        dispatcher_builder.add(
            camera::CameraSystem::default(),
            "camera_system",
            &["grid_object_system"],
        );
        // Build and setup the `Dispatcher`.
        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        grid::GridRulesSystem::deinit(world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world);
        }

        camera::CameraSystem::update_screen_dimensions(&mut data.world);
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
        grid::GridRulesSystem::deinit(world);
        grid::GridRulesSystem::init(world, self.sprites.clone().expect("sprites loaded"));
    }
}
fn load_sprites(world: &mut World) -> Handle<SpriteSheet> {
    // Load the texture for our sprites. We'll later need to
    // add a handle to this texture to our `SpriteRender`s, so
    // we need to keep a reference to it.
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "sprites/grid.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    // Load the spritesheet definition file, which contains metadata on our
    // spritesheet texture.
    let loader = world.read_resource::<Loader>();
    let sheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "sprites/grid.ron",
        SpriteSheetFormat(texture_handle),
        (),
        &sheet_storage,
    )
}
