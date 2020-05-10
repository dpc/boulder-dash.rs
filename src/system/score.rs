use amethyst::{
    assets::Loader,
    core::SystemDesc,
    derive::SystemDesc,
    ecs::prelude::*,
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};

use std::time::Instant;

use crate::grid;

pub struct Scoreboard {
    pub score: Entity,
    pub time: Entity,
}

pub fn initialise_scoreboard(world: &mut World) {
    let font_handle =
        world
            .read_resource::<Loader>()
            .load("font/bd.ttf", TtfFormat, (), &world.read_resource());

    let time_transform = UiTransform::new(
        "Score".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        -350.,
        -10.,
        1.,
        400.,
        50.,
    );
    let score_transform = UiTransform::new(
        "Time".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        200.,
        -10.,
        1.,
        400.,
        50.,
    );

    let time = world
        .create_entity()
        .with(time_transform)
        .with(UiText::new(
            font_handle.clone(),
            "0".to_string(),
            [1., 1., 1., 1.],
            50.,
        ))
        .build();

    let score = world
        .create_entity()
        .with(score_transform)
        .with(UiText::new(
            font_handle.clone(),
            "0".to_string(),
            [1., 1., 1., 1.],
            50.,
        ))
        .build();

    world.insert(Scoreboard { time, score });
}

pub fn clear_scoreboard(world: &World) {
    let scoreboard = world.read_resource::<Scoreboard>();
    let entities = world.entities();
    entities
        .delete(scoreboard.score)
        .expect("Unable to delete scoreboard score");
    entities
        .delete(scoreboard.time)
        .expect("Unable to delete scoreboard time");
}

#[derive(SystemDesc)]
pub struct ScoreSystem {
    start_time: Instant,
}

impl Default for ScoreSystem {
    fn default() -> Self {
        ScoreSystem {
            start_time: Instant::now(),
        }
    }
}

impl<'s> System<'s> for ScoreSystem {
    type SystemData = (
        WriteStorage<'s, UiText>,
        Read<'s, grid::GridState>,
        ReadExpect<'s, Scoreboard>,
    );

    fn run(&mut self, (mut ui_text, grid_map_state, score_text): Self::SystemData) {
        let time_elapsed = self.start_time.elapsed().as_secs();
        if let Some(text) = ui_text.get_mut(score_text.score) {
            text.text = format!("{:0>6}", calculate_score(grid_map_state.diamond_count));
        }

        if let Some(text) = ui_text.get_mut(score_text.time) {
            text.text = time_elapsed.to_string();
        }
    }
}

fn calculate_score(diamond_count: usize) -> usize {
    return diamond_count * 5;
}
