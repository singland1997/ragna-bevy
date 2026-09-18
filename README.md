## Asteria Meadow — a tiny Bevy 2D RPG prototype

A small, desktop-only, top‑down prototype made with Rust and Bevy that captures the spirit of classic Ragnarok‑style movement and combat without copying any assets or systems.

This is a learning/portfolio MVP, not a commercial game or RO clone.

### What’s included (MVP)
- Window titled “Asteria Meadow”
- Simple orthographic 2D tile map (grass/path/walls); walls block movement
- Player movement with WASD/arrow keys; 4‑direction facing
- One NPC who shows a dialog when you’re nearby and press E/Space
- One enemy (a slime) that damages the player on contact and can die
- Gaining XP on kill, with a basic level‑up curve (shown in HUD)
- One active skill on Q (a short‑lived projectile) with cooldown
- HUD showing HP bar, level, XP, and skill cooldown

All art is original placeholders (colored rectangles/squares generated at runtime).

### Tech
- Rust + Bevy 0.14 (documented and pinned in `Cargo.toml`)
- Bevy 2D: `Camera2d`, `SpriteBundle`, Bevy UI for HUD/dialog
- No external tilemap/physics crates; collision is grid‑based and simple

### Running
Prereqs: Rust toolchain (`rustup`, `cargo`) installed.

```bash
cargo run
```

If you just want to confirm it compiles:

```bash
cargo build
```

Tested against Bevy 0.14 on Linux. First build can take a while (Bevy compiles many crates).

### Controls
- Movement: WASD or Arrow Keys
- Interact (NPC): E or Space
- Skill: Q
- Quit: close the window normally

### Project structure
- `src/main.rs`: App setup, window, camera, and shared resources
- `src/map.rs`: Hand‑rolled tile map, wall grid, world↔map helpers
- `src/player.rs`: Player component, movement, facing, camera follow, contact damage cooldown
- `src/npc.rs`: NPC spawn and proximity interaction to open/close dialog
- `src/enemy.rs`: Enemy spawn, contact damage to player, death + XP grant
- `src/skills.rs`: One skill (projectile) with cooldown and lifetime
- `src/combat.rs`: Projectile movement and hit detection vs. enemy
- `src/ui.rs`: HUD (HP bar, Level/XP, cooldown text) + dialog overlay

### Roadmap (post‑MVP ideas)
- Better movement feel (acceleration, diagonal speed clamping, animation)
- Spritesheets and simple 4‑dir animations
- Multiple NPCs with basic branching dialog
- More enemy behaviors and damage feedback (hit flashes, numbers)
- Maps loaded from a lightweight format (RON/JSON) with editor tools
- Audio (SFX), simple music loop

Out of scope for this prototype:
- Full job/class system, inventory/economy, or long quest lines
- Networking/multiplayer
- Any use of proprietary RO/Gravity assets or branding

### Legal and attribution
This project is inspired by the feel of classic 2D RPGs like Ragnarok Online, but it is an original work using placeholder shapes and code written from scratch. It does not ship any proprietary art, maps, or other assets. The name “Asteria Meadow” is original and not affiliated with Gravity Co., Ltd.

License: MIT (see `LICENSE`).
