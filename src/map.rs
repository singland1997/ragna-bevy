use bevy::prelude::*;
use std::collections::HashSet;

pub const TILE_SIZE: f32 = 32.0;
pub const MAP_WIDTH: i32 = 48;
pub const MAP_HEIGHT: i32 = 30;

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

#[derive(Component)]
pub struct PropCollider {
    pub half_size: Vec2,
}

#[derive(Component)]
pub struct Gate;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMap>()
            .add_systems(Startup, spawn_map)
            .add_systems(Update, open_gate_when_quest_done);
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
            let is_inner_wall = (x == 16 && y >= 6 && y <= 22)
                || (y == 12 && x >= 18 && x <= 36)
                || (x == 30 && y >= 4 && y <= 18)
                || (y == 20 && x >= 8 && x <= 22);

            let tile_type = if is_border || is_inner_wall {
                TileType::Wall
            } else if y == 14 || (x >= 2 && x <= 14 && y == 8) {
                TileType::Path
            } else {
                TileType::Grass
            };

            // Simple tile color variation using a deterministic hash of (x,y)
            let v = (((x * 37 + y * 17) % 7) as f32) * 0.01;
            let color = match tile_type {
                TileType::Grass => {
                    // subtle checker: alternate slightly different green
                    let c = if (x + y) % 2 == 0 { 0.02 } else { -0.0 };
                    Color::srgb((0.12 + v + c).clamp(0.0, 1.0), (0.35 + v + c).clamp(0.0, 1.0), (0.15 + v + c).clamp(0.0, 1.0))
                }
                TileType::Path => Color::srgb(0.62 + v, 0.52 + v, 0.38 + v),
                TileType::Wall => Color::srgb(0.33 + v, 0.33 + v, 0.38 + v),
            };

            if matches!(tile_type, TileType::Wall) {
                tilemap.walls.insert(tile_coord);
            }

            let pos = map_to_world(tile_coord);
            if matches!(tile_type, TileType::Wall) {
                // Border (darker, slightly larger)
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.08, 0.08, 0.1),
                        custom_size: Some(Vec2::splat(TILE_SIZE)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(pos.x, pos.y, -0.1)),
                    ..Default::default()
                });
            }
            commands.spawn((
                Tile,
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::splat(if matches!(tile_type, TileType::Wall) { TILE_SIZE - 4.0 } else { TILE_SIZE - 1.0 })),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(pos.x, pos.y, 0.0)),
                    ..Default::default()
                },
            ));
        }
    }

    // Props: a few trees/rocks that block movement (with outlines)
    let prop_tiles = [
        IVec2::new(6, 10),
        IVec2::new(12, 16),
        IVec2::new(22, 6),
        IVec2::new(35, 20),
    ];
    for t in prop_tiles {
        let pos = map_to_world(t);
        // Outline
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.05, 0.08, 0.10),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.9, TILE_SIZE * 1.2)),
                ..Default::default()
            },
            transform: Transform::from_xyz(pos.x, pos.y + TILE_SIZE * 0.2, 3.0),
            ..Default::default()
        });
        // Body
        commands.spawn((
            PropCollider {
                half_size: Vec2::new(TILE_SIZE * 0.38, TILE_SIZE * 0.38),
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.15, 0.55, 0.18),
                    custom_size: Some(Vec2::new(TILE_SIZE * 0.78, TILE_SIZE * 1.08)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(pos.x, pos.y + TILE_SIZE * 0.22, 3.1),
                ..Default::default()
            },
            Name::new("Tree"),
        ));
    }

    // Simple gate that opens after quest completion
    let g_tile = IVec2::new(18, 12);
    let g_pos = map_to_world(g_tile);
    commands.spawn((
        Gate,
        PropCollider {
            half_size: Vec2::new(TILE_SIZE * 0.45, TILE_SIZE * 0.45),
        },
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.45, 0.38, 0.15),
                custom_size: Some(Vec2::splat(TILE_SIZE * 0.9)),
                ..Default::default()
            },
            transform: Transform::from_xyz(g_pos.x, g_pos.y, 4.0),
            ..Default::default()
        },
        Name::new("Gate"),
    ));
}

fn open_gate_when_quest_done(
    quest: Option<Res<crate::npc::QuestState>>,
    mut commands: Commands,
    q_gate: Query<Entity, With<Gate>>,
) {
    if let Some(q) = quest {
        if q.completed {
            for e in q_gate.iter() {
                commands.entity(e).despawn_recursive();
            }
        }
    }
}

