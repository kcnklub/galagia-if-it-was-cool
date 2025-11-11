mod entities;
mod input;
mod renderer;

use color_eyre::Result;
use crossterm::{
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::Rng;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::Write;
use std::time::{Duration, Instant};
use std::{fs::OpenOptions, io::stdout};

use entities::{
    Enemy, EnemyType, GameState, Pickup, Player, Projectile, ProjectileOwner, WeaponType,
};
use input::{InputAction, InputManager};
use renderer::{GameRenderer, RenderView};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let supports_keyboard_enhancement = matches!(
        crossterm::terminal::supports_keyboard_enhancement(),
        Ok(true)
    );

    // Debug: Log keyboard enhancement support
    let mut debug_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("debug.log")?;
    writeln!(
        debug_file,
        "Keyboard enhancement supported: {}",
        supports_keyboard_enhancement
    )?;

    // Setup terminal manually for full control
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Enable keyboard enhancement AFTER entering alternate screen
    if supports_keyboard_enhancement {
        execute!(
            stdout,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        )?;
        writeln!(debug_file, "Keyboard enhancement flags pushed")?;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = App::new().run(&mut terminal);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if supports_keyboard_enhancement {
        execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags)?;
    }

    terminal.show_cursor()?;

    result
}

/// The main application which holds the state and logic of the application.
pub struct App {
    /// Is the application running?
    running: bool,
    /// Current game state
    game_state: GameState,
    /// Player ship
    player: Player,
    /// Enemy ships
    enemies: Vec<Enemy>,
    /// Projectiles (from player and enemies)
    projectiles: Vec<Projectile>,
    /// Weapon pickups
    pickups: Vec<Pickup>,
    /// Player score
    score: u32,
    /// Frame counter for timing
    frame_count: u64,
    /// Last known screen dimensions
    screen_width: u16,
    screen_height: u16,
    /// Edge width to keep horizontal distance consistent
    edge_width: u16,
    /// FPS tracking
    last_frame_time: Instant,
    fps: u32,
    /// Input manager
    input_manager: InputManager,
    /// Renderer
    renderer: GameRenderer,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        // Start with reasonable defaults, will be updated on first render
        let screen_width = 120;
        let screen_height = 30;
        let edge_width = 50;

        Self {
            running: true,
            game_state: GameState::Playing,
            player: Player::new(screen_width / 2, screen_height - 5),
            enemies: Vec::new(),
            projectiles: Vec::new(),
            pickups: Vec::new(),
            score: 0,
            frame_count: 0,
            screen_width,
            screen_height,
            edge_width,
            last_frame_time: Instant::now(),
            fps: 0,
            input_manager: InputManager::new(),
            renderer: GameRenderer::new(),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        while self.running {
            // Calculate FPS
            let now = Instant::now();
            let frame_time = now.duration_since(self.last_frame_time);
            self.last_frame_time = now;
            if frame_time.as_micros() > 0 {
                self.fps = (1_000_000 / frame_time.as_micros()) as u32;
            }

            // Update screen dimensions before rendering
            let area = terminal.size()?;
            self.screen_width = area.width;
            self.screen_height = area.height;

            // Render the frame
            terminal.draw(|frame| {
                let view = RenderView {
                    game_state: self.game_state,
                    player: &self.player,
                    enemies: &self.enemies,
                    projectiles: &self.projectiles,
                    pickups: &self.pickups,
                    score: self.score,
                    frame_count: self.frame_count,
                    area: frame.area(),
                    edge_width: self.edge_width,
                    fps: self.fps,
                };
                self.renderer.render(frame, &view);
            })?;

            // Poll input events and get actions
            self.input_manager.poll_events(&self.game_state)?;
            let actions = self.input_manager.get_actions(&self.game_state);

            // Process all actions
            self.process_actions(&actions);

            // Update game state
            if self.game_state == GameState::Playing {
                self.update_game();
            }

            // Small sleep to maintain ~60 FPS and prevent CPU spinning
            std::thread::sleep(Duration::from_millis(8));
        }
        Ok(())
    }

    /// Process input actions and update game state accordingly
    fn process_actions(&mut self, actions: &[InputAction]) {
        for action in actions {
            match action {
                InputAction::Quit => {
                    self.running = false;
                }
                InputAction::Pause => {
                    self.game_state = GameState::Paused;
                }
                InputAction::Resume => {
                    self.game_state = GameState::Playing;
                }
                InputAction::Restart => {
                    *self = Self::new();
                }
                InputAction::MoveLeft => {
                    // Player coordinates are relative to game area, so min is 0
                    let min_x = 0;
                    self.player.move_left(min_x);
                }
                InputAction::MoveRight => {
                    // Max x is based on playable game area width
                    // Game area width = screen_width - (edge_width * 2) - 2 (for borders)
                    // The player occupies positions [x, x+width), so max valid x is width - player_width
                    let game_area_width = self.screen_width.saturating_sub(self.edge_width * 2 + 2);
                    // Use saturating_sub to prevent underflow, then subtract 1 more for safety
                    let max_x = game_area_width.saturating_sub(self.player.get_width() + 1);
                    self.player.move_right(max_x);
                }
                InputAction::MoveUp => {
                    let min_y = 2; // Leave space for HUD
                    self.player.move_up(min_y);
                }
                InputAction::MoveDown => {
                    let max_y = self
                        .screen_height
                        .saturating_sub(self.player.get_height() + 1);
                    self.player.move_down(max_y);
                }
                InputAction::Fire => {
                    let new_projectiles = self.player.try_fire();
                    self.projectiles.extend(new_projectiles);
                }
            }
        }
    }

    /// Update game logic
    fn update_game(&mut self) {
        self.frame_count += 1;

        // Update player cooldown
        self.player.update_cooldown();

        // Spawn enemies periodically
        if self.frame_count % 60 == 0 {
            self.spawn_enemy();
        }

        // Update projectiles
        for projectile in &mut self.projectiles {
            projectile.update();
        }

        // Remove out-of-bounds projectiles (coordinates are relative to game area)
        let game_area_width = self.screen_width.saturating_sub(self.edge_width * 2 + 2);
        self.projectiles
            .retain(|p| !p.is_out_of_bounds(0, game_area_width, self.screen_height));

        // Update enemies
        for enemy in &mut self.enemies {
            enemy.update();

            // Enemy fires occasionally
            if enemy.can_fire() && rand::thread_rng().gen_bool(0.3) {
                let enemy_width = enemy.get_width();
                let enemy_height = enemy.get_height();
                // Fire from the center bottom of the enemy sprite
                let fire_x = enemy.x + enemy_width / 2;
                let fire_y = enemy.y + enemy_height;
                self.projectiles
                    .push(Projectile::new(fire_x, fire_y, ProjectileOwner::Enemy));
            }
        }

        // Remove enemies that went off screen
        self.enemies.retain(|e| e.y < self.screen_height);

        // Spawn pickups more frequently (50% chance every 180 frames ~ every 3 seconds)
        if self.frame_count % 180 == 0 && rand::thread_rng().gen_bool(0.5) {
            self.spawn_pickup();
        }

        // Update pickups
        for pickup in &mut self.pickups {
            pickup.update();
        }

        // Remove out-of-bounds pickups
        self.pickups
            .retain(|p| !p.is_out_of_bounds(self.screen_height));

        // Check collisions
        self.check_collisions();

        // Check if player is dead
        if !self.player.is_alive() {
            self.game_state = GameState::GameOver;
        }
    }

    fn spawn_enemy(&mut self) {
        let mut rng = rand::thread_rng();
        let enemy_type = match rng.gen_range(0..10) {
            0..=5 => EnemyType::Basic,
            6..=8 => EnemyType::Fast,
            _ => EnemyType::Tank,
        };

        // Create a temporary enemy to get its width
        let temp_enemy = Enemy::new(0, 0, enemy_type);
        let enemy_width = temp_enemy.get_width();

        // Enemy coordinates are relative to game area
        // Game area width = screen_width - (edge_width * 2) - 2 (for borders)
        let game_area_width = self.screen_width.saturating_sub(self.edge_width * 2 + 2);
        let min_x = 0;
        let max_x = game_area_width.saturating_sub(enemy_width);
        let x = rng.gen_range(min_x..max_x.max(min_x + 1));

        self.enemies.push(Enemy::new(x, 2, enemy_type));
    }

    fn spawn_pickup(&mut self) {
        let mut rng = rand::thread_rng();

        // Randomly select a weapon type
        let weapon_type = match rng.gen_range(0..3) {
            0 => WeaponType::BasicGun,
            1 => WeaponType::Sword,
            _ => WeaponType::Bug,
        };

        // Pickup coordinates are relative to game area
        // Game area width = screen_width - (edge_width * 2) - 2 (for borders)
        let game_area_width = self.screen_width.saturating_sub(self.edge_width * 2 + 2);
        let min_x = 3;
        let max_x = game_area_width.saturating_sub(3);
        let x = rng.gen_range(min_x..max_x.max(min_x + 1));

        self.pickups.push(Pickup::new(x, 3, weapon_type));
    }

    fn check_collisions(&mut self) {
        // Player projectiles hitting enemies
        let mut projectiles_to_remove = Vec::new();
        let mut enemies_to_remove = Vec::new();

        for (p_idx, projectile) in self.projectiles.iter().enumerate() {
            if projectile.owner == ProjectileOwner::Player {
                for (e_idx, enemy) in self.enemies.iter_mut().enumerate() {
                    // Bounding box collision detection for larger sprites
                    let enemy_width = enemy.get_width();
                    let enemy_height = enemy.get_height();

                    if projectile.x >= enemy.x
                        && projectile.x < enemy.x + enemy_width
                        && projectile.y >= enemy.y
                        && projectile.y < enemy.y + enemy_height
                    {
                        enemy.take_damage(projectile.damage);
                        projectiles_to_remove.push(p_idx);

                        if !enemy.is_alive() {
                            self.score += enemy.get_points();
                            enemies_to_remove.push(e_idx);
                        }
                        break;
                    }
                }
            }
        }

        // Enemy projectiles hitting player
        for (p_idx, projectile) in self.projectiles.iter().enumerate() {
            if projectile.owner == ProjectileOwner::Enemy {
                let player_width = self.player.get_width();
                let player_height = self.player.get_height();

                if projectile.x >= self.player.x
                    && projectile.x < self.player.x + player_width
                    && projectile.y >= self.player.y
                    && projectile.y < self.player.y + player_height
                {
                    self.player.take_damage(projectile.damage);
                    projectiles_to_remove.push(p_idx);
                }
            }
        }

        // Enemies colliding with player
        for (e_idx, enemy) in self.enemies.iter().enumerate() {
            let enemy_width = enemy.get_width();
            let enemy_height = enemy.get_height();
            let player_width = self.player.get_width();
            let player_height = self.player.get_height();

            // Check if bounding boxes overlap
            if enemy.x < self.player.x + player_width
                && enemy.x + enemy_width > self.player.x
                && enemy.y < self.player.y + player_height
                && enemy.y + enemy_height > self.player.y
            {
                self.player.take_damage(20);
                enemies_to_remove.push(e_idx);
            }
        }

        // Remove in reverse order to avoid index issues
        projectiles_to_remove.sort_unstable();
        projectiles_to_remove.reverse();
        projectiles_to_remove.dedup();
        for idx in projectiles_to_remove {
            if idx < self.projectiles.len() {
                self.projectiles.remove(idx);
            }
        }

        enemies_to_remove.sort_unstable();
        enemies_to_remove.reverse();
        enemies_to_remove.dedup();
        for idx in enemies_to_remove {
            if idx < self.enemies.len() {
                self.enemies.remove(idx);
            }
        }

        // Player collecting pickups
        let mut pickups_to_remove = Vec::new();
        for (idx, pickup) in self.pickups.iter().enumerate() {
            let player_width = self.player.get_width();
            let player_height = self.player.get_height();
            let pickup_width = pickup.get_width();
            let pickup_height = pickup.get_height();

            // Check if bounding boxes overlap
            if pickup.x < self.player.x + player_width
                && pickup.x + pickup_width > self.player.x
                && pickup.y < self.player.y + player_height
                && pickup.y + pickup_height > self.player.y
            {
                self.player.change_weapon(pickup.weapon_type);
                pickups_to_remove.push(idx);
            }
        }

        // Remove collected pickups
        pickups_to_remove.sort_unstable();
        pickups_to_remove.reverse();
        pickups_to_remove.dedup();
        for idx in pickups_to_remove {
            if idx < self.pickups.len() {
                self.pickups.remove(idx);
            }
        }
    }
}
