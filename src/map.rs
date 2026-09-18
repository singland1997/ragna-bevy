use bevy::prelude::*;
use std::collections::HashSet;

pub const TILE_SIZE: f32 = 32.0;
pub const MAP_WIDTH: i32 = 30;
pub const MAP_HEIGHT: i32 = 20;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Grass,
    Path,
    Wall,
}

#[derive(Resource, Default)]
pub struct TileMap {
    pub width: i32,
    pub height: i32,
    pub walls: HashSet<IVec2>,
}

#[derive(Component)]
pub struct Tile;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMap>()
            .add_systems(Startup, spawn_map);
    }
}

pub fn map_to_world(tile: IVec2) -> Vec2 {
    // Center the map around (0,0)
    let origin = Vec2::new(
        -((MAP_WIDTH as f32) * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
        -((MAP_HEIGHT as f32) * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
    );
    origin + Vec2::new(tile.x as f32 * TILE_SIZE, tile.y as f32 * TILE_SIZE)
}

pub fn world_to_map(world: Vec2) -> IVec2 {
    let origin = Vec2::new(
        -((MAP_WIDTH as f32) * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
        -((MAP_HEIGHT as f32) * TILE_SIZE) / 2.0 + TILE_SIZE / 2.0,
    );
    let p = world - origin;
    IVec2::new(
        (p.x / TILE_SIZE).floor() as i32,
        (p.y / TILE_SIZE).floor() as i32,
    )
}

fn spawn_map(mut commands: Commands, mut tilemap: ResMut<TileMap>) {
    tilemap.width = MAP_WIDTH;
    tilemap.height = MAP_HEIGHT;
    tilemap.walls = HashSet::default();

    // Simple handcrafted layout: outer walls, a path, some inner walls
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let tile_coord = IVec2::new(x, y);
            let is_border = x == 0 || y == 0 || x == MAP_WIDTH - 1 || y == MAP_HEIGHT - 1;
            let is_inner_wall = (x == 10 && y >= 5 && y <= 14)
                || (y == 8 && x >= 12 && x <= 22)
                || (x == 20 && y >= 3 && y <= 10);

            let tile_type = if is_border || is_inner_wall {
                TileType::Wall
            } else if y == 10 || (x >= 2 && x <= 8 && y == 6) {
                TileType::Path
            } else {
                TileType::Grass
            };

            let color = match tile_type {
                TileType::Grass => Color::srgb(0.12, 0.35, 0.15),
                TileType::Path => Color::srgb(0.65, 0.55, 0.40),
                TileType::Wall => Color::srgb(0.35, 0.35, 0.40),
            };

            if matches!(tile_type, TileType::Wall) {
                tilemap.walls.insert(tile_coord);
            }

            let pos = map_to_world(tile_coord);
            commands.spawn((
                Tile,
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(pos.x, pos.y, 0.0)),
                    ..Default::default()
                },
            ));
        }
    }
}

