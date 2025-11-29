# Agent Guidelines for Galagia Space Battle Game

## Build/Test Commands
- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo test test_name` - Run single test
- `cargo run --release` - Run the game in release mode
- `cargo clippy` - Run linter
- `cargo fmt` - Format code

## Code Style Guidelines

### Imports & Structure
- Use `use super::*` for sibling module imports
- Re-export public types in mod.rs files
- Group imports: std, external crates, internal modules

### Naming Conventions
- Types: PascalCase (Player, EnemyType, WeaponType)
- Functions: snake_case (move_left, get_width)
- Constants: SCREAMING_SNAKE_CASE
- Fields: snake_case (fire_cooldown, current_weapon)

### Types & Patterns
- Use `u16` for coordinates, `u8` for health/cooldowns
- Derive Debug, Clone, Copy for simple enums
- Use `#[derive(Debug, Clone)]` for structs
- Implement `new()` constructors for structs

### Error Handling
- Use `color-eyre` for error handling
- Return Result<T, color_eyre::eyre::Error> from fallible functions
- Use `?` operator for error propagation

### Testing
- Write integration tests in tests/ directory
- Use descriptive test names with `test_` prefix
- Include helper functions for common operations
- Use `proptest` for property-based testing when appropriate