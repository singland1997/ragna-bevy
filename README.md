## Asteria Meadow — a tiny Bevy 2D RPG prototype

A small, desktop-only, top‑down prototype made with Rust and Bevy that captures the spirit of classic Ragnarok‑style movement and combat without copying any assets or systems.

This is a learning/portfolio MVP, not a commercial game or RO clone.

### What’s included (Phase 2)
- Window titled “Asteria Meadow”
- Orthographic 2D tile map (grass/path/walls) with simple color variation; walls + props block movement
- Player movement (WASD/arrows) with 4‑direction facing and a tiny bobbing “idle/walk” feel
- One NPC who gives a mini‑quest (“defeat 5 slimes”) via E/Space; progress shown in HUD
- Enemies chase in aggro range; contact damage on cooldown; death flash and respawn after a delay
- Melee attack on J or Left Mouse (short swing/flash + small knockback); floating damage numbers
- One active skill on Q (projectile) with cooldown
- Player death/respawn loop (brief message → respawn at start)
- HUD: clearer HP bar, XP bar, cooldown text, quest line; quick controls banner that fades

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

### Assets
- On first run, the game generates original placeholder PNGs under `assets/sprites/`:
  - `player.png`: 4× 32×32 frames in one horizontal strip (order: Up, Down, Left, Right). Used as a texture atlas for facing.
  - `npc_guide.png`: 32×32 gold NPC.
  - `slime.png`: 32×32 purple/red blob.
- These are simple high-contrast pixel sprites with dark outlines for readability and are not copied from any external game.

### Controls
- Movement: WASD or Arrow Keys
- Interact (NPC): E or Space
- Melee: J or Left Mouse
- Skill: Q (projectile)
- Quit: close the window normally

### Project structure
- `src/main.rs`: App setup, window, camera, and shared resources
- `src/map.rs`: Hand‑rolled tile map, variation, props/trees colliders, simple gate that opens after quest
- `src/player.rs`: Player component, movement + facing, camera follow, bobbing, melee cooldown, death/respawn
- `src/npc.rs`: NPC + quest state (“defeat N”), dialogue text
- `src/enemy.rs`: Enemy AI (chase), contact damage, flash/knockback, death/respawn, XP/quest progress
- `src/skills.rs`: One skill (projectile) with cooldown and lifetime
- `src/combat.rs`: Projectile + melee hit detection, knockback, floating damage numbers
- `src/ui.rs`: HUD (HP/XP bars, Level/XP/Quest, cooldown text), dialog overlay, controls hint, death overlay

### Notes and next steps
- Visuals are original shapes/outlines and simple effects; no proprietary assets.
- If you want crisp pixel art, we can add tiny PNGs and sprite flipping/animations in a follow‑up.

### Roadmap (post‑MVP ideas)
- Better movement feel (acceleration, diagonal speed clamping, animation)
- Spritesheets and simple 4‑dir animations with idle/walk cycles
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
