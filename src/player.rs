use bevy::prelude::*;
use crate::map::{self, TileMap, TILE_SIZE};

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub facing: Facing,
    pub hp: i32,
    pub max_hp: i32,
    pub contact_damage_cooldown: Timer,
}

#[derive(Clone, Copy)]
pub enum Facing {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
pub struct PlayerHitbox {
    pub half_size: Vec2,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    player_movement,
                    follow_camera.after(player_movement),
                    tick_cooldowns,
                ),
            );
    }
}

pub fn spawn_player(mut commands: Commands) {
    let start_tile = IVec2::new(5, 6);
    let start = map::map_to_world(start_tile);
    commands
        .spawn((
            Player {
                speed: 140.0,
                facing: Facing::Down,
                hp: 100,
                max_hp: 100,
                contact_damage_cooldown: Timer::from_seconds(0.6, TimerMode::Once),
            },
            PlayerHitbox {
                half_size: Vec2::new(TILE_SIZE * 0.35, TILE_SIZE * 0.35),
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.2, 0.8, 1.0),
                    custom_size: Some(Vec2::splat(TILE_SIZE * 0.8)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(start.x, start.y, 10.0),
                ..Default::default()
            },
            Name::new("Player"),
        ));
}

fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    tilemap: Res<TileMap>,
    mut q_player: Query<(&mut Transform, &mut Player, &PlayerHitbox)>,
) {
    let (mut transform, mut player, hitbox) = match q_player.get_single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };

    let mut input_dir = Vec2::ZERO;
    if keyboard.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        input_dir.y += 1.0;
    }
    if keyboard.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        input_dir.y -= 1.0;
    }
    if keyboard.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        input_dir.x -= 1.0;
    }
    if keyboard.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        input_dir.x += 1.0;
    }

    if input_dir.length_squared() > 0.0 {
        input_dir = input_dir.normalize();
        // Update facing
        player.facing = if input_dir.x.abs() > input_dir.y.abs() {
            if input_dir.x > 0.0 {
                Facing::Right
            } else {
                Facing::Left
            }
        } else {
            if input_dir.y > 0.0 {
                Facing::Up
            } else {
                Facing::Down
            }
        };
    }

    let delta = input_dir * player.speed * time.delta_seconds();
    if delta == Vec2::ZERO {
        return;
    }

    // Move with simple axis-separated collision against wall tiles
    let mut new_pos = transform.translation.truncate();

    // X axis
    new_pos.x += delta.x;
    if collides_with_walls(new_pos, hitbox.half_size, &tilemap) {
        new_pos.x -= delta.x;
    }
    // Y axis
    new_pos.y += delta.y;
    if collides_with_walls(new_pos, hitbox.half_size, &tilemap) {
        new_pos.y -= delta.y;
    }

    transform.translation.x = new_pos.x;
    transform.translation.y = new_pos.y;
}

fn collides_with_walls(pos: Vec2, half: Vec2, tilemap: &TileMap) -> bool {
    // Check the four corners
    let corners = [
        Vec2::new(pos.x - half.x, pos.y - half.y),
        Vec2::new(pos.x + half.x, pos.y - half.y),
        Vec2::new(pos.x - half.x, pos.y + half.y),
        Vec2::new(pos.x + half.x, pos.y + half.y),
    ];
    for c in corners {
        let tile = map::world_to_map(c);
        if tile.x < 0
            || tile.y < 0
            || tile.x >= tilemap.width
            || tile.y >= tilemap.height
            || tilemap.walls.contains(&tile)
        {
            return true;
        }
    }
    false
}

fn follow_camera(
    q_player: Query<&Transform, With<Player>>,
    mut q_camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let player_tf = match q_player.get_single() {
        Ok(v) => v,
        Err(_) => return,
    };
    if let Ok(mut cam_tf) = q_camera.get_single_mut() {
        cam_tf.translation.x = player_tf.translation.x;
        cam_tf.translation.y = player_tf.translation.y;
    }
}

fn tick_cooldowns(time: Res<Time>, mut q_player: Query<&mut Player>) {
    if let Ok(mut player) = q_player.get_single_mut() {
        player
            .contact_damage_cooldown
            .tick(time.delta());
    }
}

