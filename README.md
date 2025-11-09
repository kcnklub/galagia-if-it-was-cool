# Space Battle Game

A terminal-based space shooter game built with Rust and Ratatui.

## Description

A classic arcade-style space shooter where you pilot a ship and battle waves of enemies. Features smooth terminal graphics, multiple enemy types, and responsive controls.

## Features

- **Multiple Enemy Types**
  - Basic: Standard enemies (10 HP, 10 points)
  - Fast: Quick-moving enemies (5 HP, 20 points)
  - Tank: Heavy enemies (20 HP, 30 points)

- **Gameplay**
  - Smooth movement in all directions
  - Projectile combat system
  - Health tracking
  - Score system
  - Pause functionality

## Controls

- **WASD** or **Arrow Keys**: Move your ship
- **Space**: Fire projectiles
- **P**: Pause/Unpause
- **Q** or **Esc**: Quit game
- **R**: Restart (when game over)

## Installation

Ensure you have Rust installed, then:

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

## Requirements

- Rust 2024 edition
- Terminal with color support
- Recommended: Terminal size of at least 120x30

## Dependencies

- `ratatui` - Terminal UI framework
- `crossterm` - Cross-platform terminal manipulation
- `color-eyre` - Error handling
- `rand` - Random number generation

## Known Issues

- Terminal must support key repeat for smooth movement (key release events not required)
- Some terminals may experience input issues if they don't send repeated key press events

## License

Copyright (c) Kyle Miller <kylemiller457@gmail.com>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
