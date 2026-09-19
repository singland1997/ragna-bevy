use bevy::prelude::*;
use bevy::sprite::TextureAtlasLayout;
use std::fs;
use std::path::Path;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (ensure_assets_on_disk, load_assets));
    }
}

#[derive(Resource, Default, Clone)]
pub struct GameAssets {
    pub player_image: Handle<Image>,
    pub player_layout: Handle<TextureAtlasLayout>,
    pub npc_image: Handle<Image>,
    pub slime_image: Handle<Image>,
}

fn ensure_assets_on_disk() {
    let dir = Path::new("assets/sprites");
    if let Err(e) = fs::create_dir_all(dir) {
        eprintln!("Failed to create assets dir: {e}");
        return;
    }
    gen_player_if_missing(dir.join("player.png"));
    gen_npc_if_missing(dir.join("npc_guide.png"));
    gen_slime_if_missing(dir.join("slime.png"));
}

fn gen_player_if_missing(path: std::path::PathBuf) {
    if path.exists() {
        return;
    }
    // 4 frames horizontally, 32x32 each (Up, Down, Left, Right)
    let w = 32 * 4;
    let h = 32;
    let mut img = image::RgbaImage::from_pixel(w as u32, h as u32, image::Rgba([0, 0, 0, 0]));
    for i in 0..4 {
        draw_player_frame(&mut img, i * 32, 0, 32, 32, i as u32);
    }
    if let Err(e) = img.save(&path) {
        eprintln!("Failed to save {}: {e}", path.display());
    }
}

fn draw_player_frame(
    img: &mut image::RgbaImage,
    ox: i32,
    oy: i32,
    w: i32,
    h: i32,
    facing_idx: u32,
) {
    // Colors
    let outline = image::Rgba([15, 20, 26, 255]);
    let body = image::Rgba([40, 190, 255, 255]); // cyan/blue
    let head = image::Rgba([220, 245, 255, 255]);

    // Clear frame area
    for y in 0..h {
        for x in 0..w {
            let px = ox + x;
            let py = oy + y;
            img.put_pixel(px as u32, py as u32, image::Rgba([0, 0, 0, 0]));
        }
    }
    // Body rectangle with outline
    let bx0 = ox + 8;
    let by0 = oy + 14;
    let bx1 = ox + 24;
    let by1 = oy + 30;
    rect(img, bx0 - 1, by0 - 1, bx1 + 1, by1 + 1, outline);
    rect_fill(img, bx0, by0, bx1, by1, body);
    // Head circle-ish
    circle_fill(img, ox + 16, oy + 9, 7, head);
    circle(img, ox + 16, oy + 9, 8, outline);
    // Face dot to indicate direction
    let (fx, fy): (i32, i32) = match facing_idx {
        0 => (0, -5),  // Up
        1 => (0, 5),   // Down
        2 => (-5, 0),  // Left
        _ => (5, 0),   // Right
    };
    circle_fill(img, ox + 16 + fx, oy + 9 + fy, 2, outline);
}

fn gen_npc_if_missing(path: std::path::PathBuf) {
    if path.exists() {
        return;
    }
    let w = 32;
    let h = 32;
    let mut img = image::RgbaImage::from_pixel(w as u32, h as u32, image::Rgba([0, 0, 0, 0]));
    let outline = image::Rgba([26, 20, 15, 255]);
    let robe = image::Rgba([200, 150, 30, 255]); // warm gold
    // Body
    rect(&mut img, 6, 6, 26, 26, outline);
    rect_fill(&mut img, 8, 8, 24, 24, robe);
    // Hat
    rect_fill(&mut img, 10, 4, 22, 8, robe);
    rect(&mut img, 10, 4, 22, 8, outline);
    if let Err(e) = img.save(&path) {
        eprintln!("Failed to save {}: {e}", path.display());
    }
}

fn gen_slime_if_missing(path: std::path::PathBuf) {
    if path.exists() {
        return;
    }
    let w = 32;
    let h = 32;
    let mut img = image::RgbaImage::from_pixel(w as u32, h as u32, image::Rgba([0, 0, 0, 0]));
    let outline = image::Rgba([26, 10, 20, 255]);
    let body = image::Rgba([200, 40, 140, 255]); // purple/red
    // Blob
    circle_fill(&mut img, 16, 16, 11, body);
    circle(&mut img, 16, 16, 12, outline);
    // Angry eyes
    rect_fill(&mut img, 10, 14, 13, 16, outline);
    rect_fill(&mut img, 19, 14, 22, 16, outline);
    if let Err(e) = img.save(&path) {
        eprintln!("Failed to save {}: {e}", path.display());
    }
}

fn load_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let player_image: Handle<Image> = asset_server.load("sprites/player.png");
    let npc_image: Handle<Image> = asset_server.load("sprites/npc_guide.png");
    let slime_image: Handle<Image> = asset_server.load("sprites/slime.png");
    // 4 frames horizontal, 1 row
    let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 4, 1, None, None);
    let player_layout = layouts.add(layout);
    commands.insert_resource(GameAssets {
        player_image,
        player_layout,
        npc_image,
        slime_image,
    });
}

// --- tiny drawing helpers ---
fn rect(img: &mut image::RgbaImage, x0: i32, y0: i32, x1: i32, y1: i32, c: image::Rgba<u8>) {
    for x in x0..=x1 {
        if y0 >= 0 && y0 < img.height() as i32 {
            img.put_pixel(x as u32, y0 as u32, c);
        }
        if y1 >= 0 && y1 < img.height() as i32 {
            img.put_pixel(x as u32, y1 as u32, c);
        }
    }
    for y in y0..=y1 {
        if x0 >= 0 && x0 < img.width() as i32 {
            img.put_pixel(x0 as u32, y as u32, c);
        }
        if x1 >= 0 && x1 < img.width() as i32 {
            img.put_pixel(x1 as u32, y as u32, c);
        }
    }
}
fn rect_fill(img: &mut image::RgbaImage, x0: i32, y0: i32, x1: i32, y1: i32, c: image::Rgba<u8>) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            if x >= 0 && (x as u32) < img.width() && y >= 0 && (y as u32) < img.height() {
                img.put_pixel(x as u32, y as u32, c);
            }
        }
    }
}
fn circle(img: &mut image::RgbaImage, cx: i32, cy: i32, r: i32, c: image::Rgba<u8>) {
    let r2 = r * r;
    for y in -r..=r {
        for x in -r..=r {
            let d = x * x + y * y;
            if d >= r2 - 2 && d <= r2 + 2 {
                let px = cx + x;
                let py = cy + y;
                if px >= 0 && py >= 0 && (px as u32) < img.width() && (py as u32) < img.height() {
                    img.put_pixel(px as u32, py as u32, c);
                }
            }
        }
    }
}
fn circle_fill(img: &mut image::RgbaImage, cx: i32, cy: i32, r: i32, c: image::Rgba<u8>) {
    let r2 = r * r;
    for y in -r..=r {
        for x in -r..=r {
            let d = x * x + y * y;
            if d <= r2 {
                let px = cx + x;
                let py = cy + y;
                if px >= 0 && py >= 0 && (px as u32) < img.width() && (py as u32) < img.height() {
                    img.put_pixel(px as u32, py as u32, c);
                }
            }
        }
    }
}

