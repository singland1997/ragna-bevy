use bevy::prelude::*;
use bevy::sprite::TextureAtlas;
use crate::map::{self, TileMap, TILE_SIZE};
use crate::assets::GameAssets;

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

#[derive(Component)]
pub struct PlayerNameplate;

#[derive(Component)]
pub struct PlayerSprite;

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

pub fn spawn_player(mut commands: Commands, assets: Res<GameAssets>) {
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
            SpatialBundle {
                transform: Transform::from_xyz(start.x, start.y, 10.0),
                ..Default::default()
            },
            Name::new("Player"),
        ))
        .with_children(|p| {
            // Player atlas sprite
            p.spawn((
                PlayerSprite,
                SpriteBundle {
                    texture: assets.player_image.clone(),
                    transform: Transform::from_xyz(0.0, 0.0, 0.1),
                    ..Default::default()
                },
                TextureAtlas {
                    layout: assets.player_layout.clone(),
                    index: 1,
                },
                Name::new("Player Sprite"),
            ));
            // Nameplate "You"
            p.spawn((
                PlayerNameplate,
                Text2dBundle {
                    text: Text::from_section(
                        "You",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::srgb(0.7, 0.9, 1.0),
                            ..Default::default()
                        },
                    ),
                    transform: Transform::from_xyz(0.0, TILE_SIZE * 0.95, 20.0),
                    ..Default::default()
                },
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
    mut sets: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, (With<Camera>, Without<Player>)>,
    )>,
) {
    // Extract player position first to avoid overlapping ParamSet borrows
    let player_pos = {
        let q_player = sets.p0();
        match q_player.get_single() {
            Ok(tf) => tf.translation,
            Err(_) => return,
        }
    };
    if let Ok(mut cam_tf) = sets.p1().get_single_mut() {
        cam_tf.translation.x = player_pos.x;
        cam_tf.translation.y = player_pos.y;
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
    mut q_sprite: Query<(&mut Transform, &mut TextureAtlas), With<PlayerSprite>>,
) {
    let (player, children) = match q_player.get_single() {
        Ok(v) => v,
        Err(_) => return,
    };
    // Set atlas index by facing: 0=Up,1=Down,2=Left,3=Right (matches generator)
    let idx = match player.facing {
        Facing::Up => 0,
        Facing::Down => 1,
        Facing::Left => 2,
        Facing::Right => 3,
    };
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
        if let Ok((mut tf, mut atlas)) = q_sprite.get_mut(child) {
            tf.translation.y = bob;
            atlas.index = idx;
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

