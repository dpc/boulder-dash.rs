use rand::{thread_rng, Rng};

use crate::{
    input::{self, Direction},
    map::MapDescription,
};

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct GridPos(usize);

/*
impl GridPos {
    fn to_translation(self) -> Vector3<f32> {
        Vector3::new(self.x as f32 * TILE_SIZE, self.y as f32 * TILE_SIZE, 0.)
    }
}
*/

impl GridPos {
    pub fn new(i: usize) -> Self {
        Self(i)
    }

    pub fn from_xy(x: usize, y: usize, width: usize) -> Self {
        Self(x + y * width)
    }

    pub fn direction(self, d: input::Direction, width: usize) -> Self {
        use Direction::*;
        match d {
            Up => self.up(width),
            Down => self.down(width),
            Left => self.left(),
            Right => self.right(),
        }
    }

    pub fn to_xy(self, width: usize) -> (usize, usize) {
        let x = self.0 % width;
        let y = self.0 / width;

        (x, y)
    }

    pub fn up(self, width: usize) -> Self {
        Self(self.0 + width)
    }

    pub fn down(self, width: usize) -> Self {
        Self(self.0 - width)
    }
    pub fn left(self) -> Self {
        Self(self.0 - 1)
    }
    pub fn right(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Tile {
    pos: GridPos,
    kind: TileType,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TileType {
    Empty,
    Player,
    Dirt,
    Rock,
    Wall,
    Diamond,
    Steel,
    Creature {
        counter: usize,
        direction: Direction,
    },
}

impl Default for TileType {
    fn default() -> Self {
        TileType::Empty
    }
}

impl TileType {
    pub fn to_sprite_number(self) -> Option<usize> {
        use TileType::*;
        Some(match self {
            Empty => return None,
            Player => 0,
            Dirt => 2,
            Rock => 3,
            Steel => 4,
            Wall => 5,
            Diamond => 6,
            Creature {
                counter: _,
                direction: _,
            } => 7,
        })
    }

    fn can_fall(self) -> bool {
        use TileType::*;
        match self {
            Rock => true,
            Diamond => true,
            _ => false,
        }
    }

    fn can_be_rolled_on(self) -> bool {
        use TileType::*;
        match self {
            Rock => true,
            Diamond => true,
            Wall => true,
            _ => false,
        }
    }

    fn can_be_bounced_on(self) -> bool {
        use TileType::*;
        match self {
            Player => false,
            _ => true,
        }
    }

    fn can_be_stepped_on(self) -> bool {
        use TileType::*;
        match self {
            Empty => true,
            Dirt => true,
            Diamond => true,
            _ => false,
        }
    }

    fn is_empty(self) -> bool {
        use TileType::*;
        match self {
            Empty => true,
            _ => false,
        }
    }

    fn is_player(self) -> bool {
        use TileType::*;
        match self {
            Player => true,
            _ => false,
        }
    }

    fn is_diamond(self) -> bool {
        use TileType::*;
        match self {
            Diamond => true,
            _ => false,
        }
    }

    fn can_be_pushed(self) -> bool {
        use TileType::*;
        match self {
            Rock => true,
            _ => false,
        }
    }
}

#[derive(Default, Clone)]
pub struct GridState {
    height: usize,
    width: usize,
    pub tiles: Vec<TileType>,

    pub player_pos: GridPos,

    // TODO: move?
    pub diamond_count: usize,
}

impl GridState {
    pub fn new() -> Self {
        let MapDescription {
            height,
            width,
            tiles,
            start,
        } = crate::map::MapDescription::load("./resources/map/01.txt".into())
            .expect("map should load");

        GridState {
            height,
            width,
            tiles,
            player_pos: start,
            ..GridState::default()
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn get_tile(&self, pos: GridPos) -> TileType {
        *self.get_tile_ref(pos)
    }

    pub fn get_tile_relative(&self, pos: GridPos, direction: Direction) -> Tile {
        let dst_pos = pos.direction(direction, self.width);
        let dst_type = self.get_tile(dst_pos);
        Tile {
            pos: dst_pos,
            kind: dst_type,
        }
    }

    pub fn set_tile(&mut self, pos: GridPos, v: TileType) {
        *self.get_tile_mut(pos) = v;
    }

    pub fn get_tile_ref(&self, pos: GridPos) -> &TileType {
        &self.tiles[pos.0]
    }
    pub fn get_tile_mut(&mut self, pos: GridPos) -> &mut TileType {
        &mut self.tiles[pos.0]
    }

    pub fn run_tick(&mut self, action: Vec<input::Action>) {
        self.move_player(action);

        for i in 0..self.tiles.len() {
            let pos = GridPos(i);
            if let Some(new_tile) = self.run_tick_for_pos(pos) {
                self.move_grid_object(pos, new_tile.pos);
                // sometimes the tick will change the kind (i.e. creature direction / counter)
                self.set_tile(new_tile.pos, new_tile.kind);
            }
        }
    }

    // return the new position of the tile, or none if it didn't move
    fn run_tick_for_pos(&mut self, current_pos: GridPos) -> Option<Tile> {
        let tile = self.get_tile(current_pos);
        if tile.can_fall() {
            if let Some(new_pos) = self.try_roll(current_pos) {
                return Some(Tile {
                    pos: new_pos,
                    kind: tile,
                });
            }

            if let Some(new_pos) = self.try_fall(current_pos) {
                return Some(Tile {
                    pos: new_pos,
                    kind: tile,
                });
            }
        }

        if let TileType::Creature { counter, direction } = tile {
            // counter just to slow down movement of the creatures
            if counter % 3 == 2 {
                if let Some(new_tile) = self.try_creature_move(current_pos, direction) {
                    return Some(new_tile);
                }
            } else {
                return Some(Tile {
                    pos: current_pos,
                    kind: TileType::Creature {
                        counter: counter + 1,
                        direction: direction,
                    },
                });
            }
        }

        None
    }

    // move something from src_pos to dst_pos
    // anything at dst_pos will be destroyed
    fn move_grid_object(&mut self, src_pos: GridPos, dst_pos: GridPos) {
        let src_type = self.get_tile(src_pos);
        self.set_tile(src_pos, TileType::Empty);
        self.set_tile(dst_pos, src_type);

        if self.player_pos == src_pos {
            assert_eq!(src_type, TileType::Player);
            self.player_pos = dst_pos;
        }
    }

    fn move_player(&mut self, action: Vec<input::Action>) {
        let player_pos = self.player_pos;

        debug_assert!(self.get_tile(self.player_pos).is_player());

        for action in action {
            let dst = self.get_tile_relative(player_pos, action.direction);
            if dst.kind.is_diamond() {
                self.diamond_count += 1;
            }
            if dst.kind.can_be_stepped_on() {
                self.move_grid_object(player_pos, dst.pos);
                break;
            }
            if dst.kind.can_be_pushed() {
                let past_dst = self.get_tile_relative(dst.pos, action.direction);
                if past_dst.kind.is_empty() {
                    self.move_grid_object(dst.pos, past_dst.pos);
                    self.move_grid_object(player_pos, dst.pos);
                    break;
                }
            }
        }
    }

    fn try_roll(&mut self, current_pos: GridPos) -> Option<GridPos> {
        let down = self.get_tile_relative(current_pos, Direction::Down);
        if !down.kind.can_be_rolled_on() {
            return None;
        }

        let left = self.get_tile_relative(current_pos, Direction::Left);
        let down_left = self.get_tile_relative(left.pos, Direction::Down);
        let right = self.get_tile_relative(current_pos, Direction::Right);
        let down_right = self.get_tile_relative(right.pos, Direction::Down);

        let left_free = left.kind.is_empty() && down_left.kind.is_empty();
        let right_free = right.kind.is_empty() && down_right.kind.is_empty();

        match (left_free, right_free) {
            (true, true) => {
                let mut rng = thread_rng();
                let choices = [left.pos, right.pos];
                Some(choices[rng.gen_range(0, choices.len())])
            }
            (true, false) => Some(left.pos),
            (false, true) => Some(right.pos),
            (false, false) => None,
        }
    }

    fn try_fall(&mut self, current_pos: GridPos) -> Option<GridPos> {
        let below = self.get_tile_relative(current_pos, Direction::Down);

        if below.kind.is_empty() {
            Some(below.pos)
        } else {
            None
        }
    }

    fn try_creature_move(&mut self, current_pos: GridPos, direction: Direction) -> Option<Tile> {
        let front = self.get_tile_relative(current_pos, direction);
        if front.kind.is_empty() {
            return Some(Tile {
                pos: front.pos,
                kind: TileType::Creature {
                    counter: 0,
                    direction: direction,
                },
            });
        }

        if !front.kind.can_be_bounced_on() {
            return None;
        }

        // randomly pick from any open side, preferring not to switch all the way around
        let choices = [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ];
        let filtered: Vec<_> = choices
            .iter()
            .filter(|d| {
                **d != direction.opposite()
                    && self.get_tile_relative(current_pos, **d).kind == TileType::Empty
            })
            .collect();

        if filtered.len() > 0 {
            let mut rng = thread_rng();
            let new_direction = *filtered[rng.gen_range(0, filtered.len())];
            let new_tile = self.get_tile_relative(current_pos, new_direction);

            return Some(Tile {
                pos: new_tile.pos,
                kind: TileType::Creature {
                    counter: 0,
                    direction: new_direction,
                },
            });
        }

        let opposite = self.get_tile_relative(current_pos, direction.opposite());
        match opposite.kind {
            TileType::Empty => Some(Tile {
                pos: opposite.pos,
                kind: TileType::Creature {
                    counter: 0,
                    direction: direction.opposite(),
                },
            }),
            _ => None,
        }
    }
}
