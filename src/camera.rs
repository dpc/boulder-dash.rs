use amethyst::{
    core::transform::Transform,
    core::SystemDesc,
    core::Time,
    derive::SystemDesc,
    ecs::{prelude::*, Read, System, SystemData, World, WriteStorage},
    renderer::Camera,
    window::ScreenDimensions,
};

use crate::grid;
use crate::TILE_SIZE;

#[derive(Debug, Copy, Clone)]
pub struct ZoomLevel(f32);

impl std::default::Default for ZoomLevel {
    fn default() -> Self {
        ZoomLevel(1.)
    }
}

impl ZoomLevel {
    fn new() -> Self {
        ZoomLevel(1.)
    }
    pub fn up(self) -> Self {
        Self(clamp(self.0 * 2., 0.25, 8.))
    }

    pub fn down(self) -> Self {
        Self(clamp(self.0 * 0.5, 0.25, 8.))
    }
}

#[derive(SystemDesc, Debug, Default)]
pub struct CameraSystem {
    current_x: f32,
    current_y: f32,
}

impl CameraSystem {
    pub fn init(world: &mut World, dimensions: &ScreenDimensions) {
        let camera_x = dimensions.width();
        let camera_y = dimensions.height();

        // Center the camera in the middle of the screen, and let it cover
        // the entire screen
        let transform = Transform::default();
        // transform.set_translation_xyz(camera_x * 0.5, camera_y * 0.5, 1.);

        world
            .create_entity()
            .with(Camera::standard_2d(camera_x, camera_y))
            .with(transform)
            .build();

        world.insert(ZoomLevel::new());

        Self::update_screen_dimensions(world);
    }

    pub fn update_screen_dimensions(world: &mut World) {
        let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();
        world.insert(Some(dimensions));
    }
}

fn clamp(v: f32, low: f32, high: f32) -> f32 {
    if low > high {
        (low + high) / 2.
    } else if v < low {
        low
    } else if high < v {
        high
    } else {
        v
    }
}

impl<'s> System<'s> for CameraSystem {
    type SystemData = (
        WriteStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        Read<'s, grid::GridState>,
        Read<'s, Option<ScreenDimensions>>,
        Read<'s, ZoomLevel>,
    );

    fn run(
        &mut self,
        (mut camera, mut transform, time, grid_map_state, screen_dimensions, zoom_level): Self::SystemData,
    ) {
        let screen_dimensions = screen_dimensions.as_ref().expect("screen dimmensions set");

        let screen_w = screen_dimensions.width();
        let screen_h = screen_dimensions.height();
        let zoom_level = (*zoom_level).0;
        let camera_w = screen_w / zoom_level;
        let camera_h = screen_h / zoom_level;
        let padding_x = camera_w / 2.;
        let padding_y = camera_h / 2.;

        let desired_x = clamp(
            grid_map_state.player_pos.x as f32 * TILE_SIZE,
            padding_x,
            (grid_map_state.tiles.width().saturating_sub(1) as f32 * TILE_SIZE) - padding_x,
        );
        let desired_y = clamp(
            grid_map_state.player_pos.y as f32 * TILE_SIZE,
            padding_y,
            (grid_map_state.tiles.height().saturating_sub(1) as f32 * TILE_SIZE) - padding_y,
        );
        // let desired_x = screen_w / 2.;
        // let desired_y = screen_h / 2.;

        let desired_camera = Camera::standard_2d(
            // tiles_width_to_show * 32.,
            // tiles_width_to_show * 32. * screen_h / screen_w,
            camera_w, camera_h,
        );
        for (camera, transform) in (&mut camera, &mut transform).join() {
            *camera = desired_camera.clone();
            // let current_translation = transform.translation().clone();
            let max_dv = 3. * TILE_SIZE * time.delta_real_seconds();
            let dx = (desired_x - self.current_x).max(-max_dv).min(max_dv);
            let dy = (desired_y - self.current_y).max(-max_dv).min(max_dv);
            self.current_x += dx;
            self.current_y += dy;
            transform.set_translation_xyz(self.current_x.round(), self.current_y.round(), 1.0);
        }
    }
}
