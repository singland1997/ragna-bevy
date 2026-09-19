use bevy::prelude::*;
use crate::map::{self, TileMap, TILE_SIZE};

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub facing: Facing,
    pub hp: i32,
    pub max_hp: i32,
    pub contact_damage_cooldown: Timer,
    pub melee_cooldown: Timer,
    pub alive: bool,
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

#[derive(Component)]
pub struct PlayerBody; // visual parent for body

#[derive(Component)]
pub struct PlayerFace; // small dot indicating facing

#[derive(Resource, Default)]
struct PlayerStart {
    pos: Vec2,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerStart>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    player_movement,
                    follow_camera.after(player_movement),
                    update_player_visual.after(player_movement),
                    tick_cooldowns,
                    handle_player_death_and_respawn,
                ),
            );
    }
}

pub fn spawn_player(mut commands: Commands) {
    let start_tile = IVec2::new(5, 6);
    let start = map::map_to_world(start_tile);
    // Save start for respawns
    commands.insert_resource(PlayerStart { pos: start });
    commands
        .spawn((
            Player {
                speed: 140.0,
                facing: Facing::Down,
                hp: 100,
                max_hp: 100,
                contact_damage_cooldown: Timer::from_seconds(0.6, TimerMode::Once),
                melee_cooldown: Timer::from_seconds(0.35, TimerMode::Once),
                alive: true,
            },
            PlayerHitbox {
                half_size: Vec2::new(TILE_SIZE * 0.35, TILE_SIZE * 0.35),
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::NONE,
                    custom_size: Some(Vec2::splat(TILE_SIZE * 0.01)), // invisible parent
                    ..Default::default()
                },
                transform: Transform::from_xyz(start.x, start.y, 10.0),
                ..Default::default()
            },
            Name::new("Player"),
        ))
        .with_children(|p| {
            // Outline (slightly bigger dark sprite)
            p.spawn((
                PlayerBody,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.05, 0.08, 0.10),
                        custom_size: Some(Vec2::splat(TILE_SIZE * 0.9)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..Default::default()
                },
                Name::new("Player Outline"),
            ));
            // Body
            p.spawn((
                PlayerBody,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.2, 0.8, 1.0),
                        custom_size: Some(Vec2::splat(TILE_SIZE * 0.78)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 0.1),
                    ..Default::default()
                },
                Name::new("Player Body"),
            ));
            // Face dot to indicate facing
            p.spawn((
                PlayerFace,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::WHITE,
                        custom_size: Some(Vec2::splat(TILE_SIZE * 0.18)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(0.0, -TILE_SIZE * 0.18, 0.2),
                    ..Default::default()
                },
                Name::new("Player Face"),
            ));
        });
}

fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    tilemap: Res<TileMap>,
    // props collider check optional via Without<Player>
    props: Query<(&Transform, &crate::map::PropCollider), Without<Player>>,
    death: Res<crate::DeathState>,
    mut q_player: Query<(&mut Transform, &mut Player, &PlayerHitbox)>,
) {
    if death.dead {
        return;
    }
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
    if collides_with_walls(new_pos, hitbox.half_size, &tilemap)
        || collides_with_props(new_pos, hitbox.half_size, &props)
    {
        new_pos.x -= delta.x;
    }
    // Y axis
    new_pos.y += delta.y;
    if collides_with_walls(new_pos, hitbox.half_size, &tilemap)
        || collides_with_props(new_pos, hitbox.half_size, &props)
    {
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

fn collides_with_props(
    pos: Vec2,
    half: Vec2,
    props: &Query<(&Transform, &crate::map::PropCollider), Without<Player>>,
) -> bool {
    for (tf, prop) in props.iter() {
        if aabb_overlap(pos, half, tf.translation.truncate(), prop.half_size) {
            return true;
        }
    }
    false
}

fn aabb_overlap(a_pos: Vec2, a_half: Vec2, b_pos: Vec2, b_half: Vec2) -> bool {
    (a_pos.x - b_pos.x).abs() <= (a_half.x + b_half.x)
        && (a_pos.y - b_pos.y).abs() <= (a_half.y + b_half.y)
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
        player.melee_cooldown.tick(time.delta());
    }
}

fn update_player_visual(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    q_player: Query<(&Player, &Children)>,
    mut q_face: Query<&mut Transform, With<PlayerFace>>,
    mut q_body: Query<&mut Transform, (With<PlayerBody>, Without<PlayerFace>)>,
) {
    let (player, children) = match q_player.get_single() {
        Ok(v) => v,
        Err(_) => return,
    };
    // Move face dot based on facing
    let offset = match player.facing {
        Facing::Up => Vec2::new(0.0, TILE_SIZE * 0.18),
        Facing::Down => Vec2::new(0.0, -TILE_SIZE * 0.18),
        Facing::Left => Vec2::new(-TILE_SIZE * 0.18, 0.0),
        Facing::Right => Vec2::new(TILE_SIZE * 0.18, 0.0),
    };
    for &child in children.iter() {
        if let Ok(mut tf) = q_face.get_mut(child) {
            tf.translation.x = offset.x;
            tf.translation.y = offset.y;
        }
    }
    // Simple bobbing when any movement key is pressed
    let moving = keyboard.any_pressed([
        KeyCode::KeyW,
        KeyCode::KeyA,
        KeyCode::KeyS,
        KeyCode::KeyD,
        KeyCode::ArrowUp,
        KeyCode::ArrowLeft,
        KeyCode::ArrowDown,
        KeyCode::ArrowRight,
    ]);
    let bob = if moving {
        (time.elapsed_seconds() * 10.0).sin() * 1.5
    } else {
        0.0
    };
    for &child in children.iter() {
        if let Ok(mut tf) = q_body.get_mut(child) {
            tf.translation.y = bob;
        }
    }
}

fn handle_player_death_and_respawn(
    time: Res<Time>,
    mut death: ResMut<crate::DeathState>,
    start: Res<PlayerStart>,
    mut q_player: Query<(&mut Player, &mut Transform)>,
) {
    let (mut player, mut tf) = match q_player.get_single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };
    if !death.dead && player.hp <= 0 {
        death.dead = true;
        death.timer.reset();
    }
    if death.dead {
        death.timer.tick(time.delta());
        if death.timer.finished() {
            // respawn
            player.hp = player.max_hp;
            tf.translation.x = start.pos.x;
            tf.translation.y = start.pos.y;
            death.dead = false;
        }
    }
}

