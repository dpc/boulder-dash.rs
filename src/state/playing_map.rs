use amethyst::{
    assets::Handle,
    core::{math::Vector3, Transform},
    ecs::{prelude::*, NullStorage},
    input::{get_key, is_close_requested, is_key_down, ElementState, VirtualKeyCode},
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
    window::ScreenDimensions,
};

use crate::{camera, grid, input};

#[derive(Default)]
pub struct PlayingMap<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
    sprites: Option<Handle<SpriteSheet>>,
    tick_count: u64,
    input_tracker: input::InputTracker,
}

impl<'a, 'b> SimpleState for PlayingMap<'a, 'b> {
    // On start will run when this state is initialized. For more
    // state lifecycle hooks, see:
    // https://book.amethyst.rs/stable/concepts/state.html#life-cycle
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.insert(grid::GridState::new());

        // Get the screen dimensions so we can initialize the camera and
        // place our sprites correctly later. We'll clone this since we'll
        // pass the world mutably to the following functions.
        let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();
        world.register::<GridSprite>();

        // Load our sprites and display them
        let sprites = super::load_sprites(world);
        self.sprites = Some(sprites);
        camera::CameraSystem::init(world, &dimensions);

        // Create the `DispatcherBuilder` and register some `System`s that should only run for this `State`.
        let mut dispatcher_builder = DispatcherBuilder::new();

        dispatcher_builder.add(camera::CameraSystem::default(), "camera_system", &[]);
        // Build and setup the `Dispatcher`.
        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);
    }

    fn on_stop(&mut self, _data: StateData<'_, GameData<'_, '_>>) {}

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(&data.world);
        }

        camera::CameraSystem::update_screen_dimensions(&mut data.world);
        SimpleTrans::None
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        self.tick_count += 1;
        if self.tick_count % 8 == 0 {
            {
                let mut grid = data.world.write_resource::<crate::grid::GridState>();
                grid.run_tick(self.input_tracker.pop_action());
            }
            self.redraw_grid(data);
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
                    event => self.input_tracker.handle_key(event),
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

#[derive(Copy, Clone, Default, Debug)]
struct GridSprite;

impl Component for GridSprite {
    type Storage = NullStorage<Self>;
}

impl<'a, 'b> PlayingMap<'a, 'b> {
    fn restart_map(&mut self, world: &mut World) {
        world.insert(grid::GridState::new());
    }

    fn clear_grid(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) {
        let world = &mut data.world;
        {
            let entities = world.entities();

            for (entity, _sprite) in (&entities, &world.read_storage::<GridSprite>()).join() {
                entities.delete(entity).expect("to work");
            }
        }
        world.maintain();
    }

    fn redraw_grid(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        self.clear_grid(&mut data);
        // TODO: lame clone
        let grid = (*data.world.read_resource::<crate::grid::GridState>()).clone();

        // TODO: we don't need to draw all sprites - only the ones that are near
        // TODO: maybe reusing entities is faster?
        for i in 0..(grid.width() * grid.height()) {
            let pos = grid::GridPos::new(i);
            let tile = grid.get_tile(pos);
            if let Some(sprite_number) = tile.to_sprite_number() {
                let sprite_render = SpriteRender {
                    sprite_sheet: self.sprites.clone().expect("sprites already loaded"),
                    sprite_number,
                };
                let (x, y) = pos.to_xy(grid.width());

                let mut transform = Transform::default();
                transform.set_translation(Vector3::new(
                    x as f32 * crate::TILE_SIZE,
                    y as f32 * crate::TILE_SIZE,
                    0.,
                ));
                let _entity = data
                    .world
                    .create_entity()
                    .with(sprite_render)
                    .with(transform)
                    .with(GridSprite)
                    .build();
            }
        }
    }
}
