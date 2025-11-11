use color_eyre::Result;
use rand::Rng;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::time::{Duration, Instant};

use crate::audio::AudioManager;
use crate::entities::{
    Enemy, EnemyType, Formation, FormationType, GameState, Particle, Pickup, Player, Projectile,
    ProjectileOwner, ProjectileType, WeaponType, create_explosion_particles,
};
use crate::input::{InputAction, InputManager};
use crate::renderer::{GameRenderer, RenderView};

/// The main application which holds the state and logic of the application.
pub struct App {
    running: bool,
    game_state: GameState,
    player: Player,
    enemies: Vec<Enemy>,
    formations: Vec<Formation>,
    /// Projectiles (from player and enemies)
    projectiles: Vec<Projectile>,
    particles: Vec<Particle>,
    pickups: Vec<Pickup>,
    score: u32,
    /// screen dimensions
    screen_width: u16,
    screen_height: u16,
    edge_width: u16,
    /// Frames info
    frame_count: u64,
    spawn_delay_frames: u64,
    last_frame_time: Instant,
    fps: u32,
    /// Game timers
    game_start_time: Instant,
    final_time_secs: Option<u64>,
    /// internal components
    input_manager: InputManager,
    renderer: GameRenderer,
    audio_manager: AudioManager,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        // Start with reasonable defaults, will be updated on first render
        let screen_width: u16 = 60;
        let screen_height: u16 = 70;
        let edge_width: u16 = 50;

        // Center player horizontally in the screen, position near bottom vertically
        let player_x = edge_width + screen_width / 2; // 6 units from bottom
        let player_y = screen_height - (screen_height / 5); // Center horizontally on screen

        let now = Instant::now();
        let mut app = Self {
            running: true,
            game_state: GameState::Playing,
            player: Player::new(player_x, player_y),
            enemies: Vec::new(),
            formations: Vec::new(),
            projectiles: Vec::new(),
            particles: Vec::new(),
            pickups: Vec::new(),
            score: 0,
            frame_count: 0,
            screen_width,
            screen_height,
            edge_width,
            spawn_delay_frames: 0,
            last_frame_time: now,
            fps: 0,
            game_start_time: now,
            final_time_secs: None,
            input_manager: InputManager::new(),
            renderer: GameRenderer::new(),
            audio_manager: AudioManager::default(),
        };

        // Spawn initial formation so player doesn't have to wait
        app.spawn_formation();

        app
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
                // Use final time if game is over, otherwise calculate current elapsed time
                let elapsed_time_secs = self
                    .final_time_secs
                    .unwrap_or_else(|| self.game_start_time.elapsed().as_secs());
                let view = RenderView {
                    game_state: self.game_state,
                    player: &self.player,
                    enemies: &self.enemies,
                    projectiles: &self.projectiles,
                    particles: &self.particles,
                    pickups: &self.pickups,
                    score: self.score,
                    frame_count: self.frame_count,
                    area: frame.area(),
                    edge_width: self.edge_width,
                    fps: self.fps,
                    elapsed_time_secs,
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
                    if !new_projectiles.is_empty() {
                        self.audio_manager.play_fire_sound();
                    }
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

        // Check if all enemies are dead and spawn new formation after delay
        if self.enemies.is_empty() {
            if self.spawn_delay_frames > 0 {
                self.spawn_delay_frames -= 1;
            } else {
                // Spawn new formation
                self.spawn_formation();
                // Set delay for next spawn (90 frames = ~1.5 seconds at 60 FPS)
                self.spawn_delay_frames = 90;
            }
        } else {
            // Reset delay counter if there are enemies alive
            self.spawn_delay_frames = 90;
        }

        // Update projectiles
        for projectile in &mut self.projectiles {
            projectile.update();
        }

        // Remove out-of-bounds projectiles (coordinates are relative to game area)
        let game_area_width = self.screen_width.saturating_sub(self.edge_width * 2 + 2);
        self.projectiles
            .retain(|p| !p.is_out_of_bounds(0, game_area_width, self.screen_height));

        // Update particles
        for particle in &mut self.particles {
            particle.update();
        }

        // Remove dead or out-of-bounds particles
        self.particles.retain(|p| {
            !p.is_dead() && !p.is_out_of_bounds(0, game_area_width, self.screen_height)
        });

        // Update formations
        let game_area_width = self.screen_width.saturating_sub(self.edge_width * 2 + 2);
        for formation in &mut self.formations {
            formation.update(game_area_width);
        }

        // Update enemy positions based on formations
        for enemy in self.enemies.iter_mut() {
            if let Some(formation_id) = enemy.formation_id
                && formation_id < self.formations.len() {
                    let formation = &self.formations[formation_id];
                    enemy.update_formation_position(formation.center_x, formation.center_y);
                }

            enemy.update();

            if enemy.can_fire() && rand::rng().random_bool(0.3) {
                let enemy_width = enemy.get_width();
                let enemy_height = enemy.get_height();
                // Fire from the center bottom of the enemy sprite
                let fire_x = enemy.x + enemy_width / 2;
                let fire_y = enemy.y + enemy_height;
                self.projectiles
                    .push(Projectile::new(fire_x, fire_y, ProjectileOwner::Enemy));
                self.audio_manager.play_fire_sound_volume(0.01);
            }
        }

        // Remove enemies that went off screen
        self.enemies.retain(|e| e.y < self.screen_height);

        // Clean up formations that have no enemies left or went off screen
        self.formations.retain(|f| {
            f.center_y < self.screen_height
                && f.enemy_indices
                    .iter()
                    .any(|&idx| idx < self.enemies.len() && self.enemies[idx].is_alive())
        });

        // Spawn pickups more frequently (50% chance every 180 frames ~ every 3 seconds)
        if self.frame_count.is_multiple_of(180) && rand::rng().random_bool(0.5) {
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
            // Capture final time when transitioning to game over
            if self.game_state != GameState::GameOver {
                self.final_time_secs = Some(self.game_start_time.elapsed().as_secs());
            }
            self.game_state = GameState::GameOver;
        }
    }

    fn spawn_formation(&mut self) {
        let mut rng = rand::rng();

        // Randomly select a formation type
        let formation_type = match rng.random_range(0..4) {
            0 => FormationType::VShape,
            1 => FormationType::Diamond,
            2 => FormationType::Wall,
            _ => FormationType::Block,
        };

        // Calculate game area
        let game_area_width = self.screen_width.saturating_sub(self.edge_width * 2 + 2);

        // Place formation center somewhere in the upper third of the screen
        // Add some padding from edges (30 units on each side)
        let min_x = 30;
        let max_x = game_area_width.saturating_sub(30);
        let center_x = rng.random_range(min_x..max_x.max(min_x + 1));
        let center_y = 5;

        let formation_id = self.formations.len();
        let mut formation = Formation::new(center_x, center_y, formation_type);

        // Get positions and create enemies
        let positions = formation.get_positions();
        let enemy_type = match rng.random_range(0..10) {
            0..=6 => EnemyType::Basic,
            7..=8 => EnemyType::Fast,
            _ => EnemyType::Tank,
        };

        for offset in positions {
            let x = (center_x as i16 + offset.0).max(0) as u16;
            let y = (center_y as i16 + offset.1).max(0) as u16;

            let enemy_idx = self.enemies.len();
            formation.enemy_indices.push(enemy_idx);

            self.enemies.push(Enemy::new_in_formation(
                x,
                y,
                enemy_type,
                formation_id,
                offset,
            ));
        }

        self.formations.push(formation);
    }

    fn spawn_pickup(&mut self) {
        let mut rng = rand::rng();

        // Randomly select a weapon type
        let weapon_type = match rng.random_range(0..4) {
            0 => WeaponType::BasicGun,
            1 => WeaponType::Sword,
            2 => WeaponType::Bug,
            _ => WeaponType::Bomber,
        };

        // Pickup coordinates are relative to game area
        // Game area width = screen_width - (edge_width * 2) - 2 (for borders)
        let game_area_width = self.screen_width.saturating_sub(self.edge_width * 2 + 2);
        let min_x = 3;
        let max_x = game_area_width.saturating_sub(3);
        let x = rng.random_range(min_x..max_x.max(min_x + 1));

        self.pickups.push(Pickup::new(x, 3, weapon_type));
    }

    fn check_collisions(&mut self) {
        // Player projectiles hitting enemies
        let mut projectiles_to_remove = Vec::new();
        let mut enemies_to_remove = Vec::new();

        for (p_idx, projectile) in self.projectiles.iter().enumerate() {
            if projectile.owner == ProjectileOwner::Player {
                // Check if bomber projectile lifetime expired (explodes)
                if projectile.projectile_type == ProjectileType::BomberProjectile
                    && projectile.lifetime == Some(0)
                {
                    // Explosion! Deal AoE damage to all enemies in radius
                    const EXPLOSION_RADIUS: u16 = 8;
                    const EXPLOSION_DAMAGE: u8 = 25;

                    // Create explosion particle effect
                    let explosion_particles =
                        create_explosion_particles(projectile.x, projectile.y);
                    self.particles.extend(explosion_particles);

                    for (e_idx, enemy) in self.enemies.iter_mut().enumerate() {
                        // Calculate distance between explosion center and enemy center
                        let enemy_center_x = enemy.x + enemy.get_width() / 2;
                        let enemy_center_y = enemy.y + enemy.get_height() / 2;

                        let dx = (projectile.x as i32 - enemy_center_x as i32).abs();
                        let dy = (projectile.y as i32 - enemy_center_y as i32).abs();

                        // Simple circle collision (using squared distance to avoid sqrt)
                        if (dx * dx + dy * dy)
                            <= (EXPLOSION_RADIUS as i32 * EXPLOSION_RADIUS as i32)
                        {
                            enemy.take_damage(EXPLOSION_DAMAGE);

                            if !enemy.is_alive() {
                                // Create particles at enemy death location
                                let death_particles =
                                    create_explosion_particles(enemy_center_x, enemy_center_y);
                                self.particles.extend(death_particles);

                                self.score += enemy.get_points();
                                enemies_to_remove.push(e_idx);
                            }
                        }
                    }
                    projectiles_to_remove.push(p_idx);
                    continue;
                }

                // Regular collision detection for non-bomber projectiles
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
                            // Create particles at enemy death location
                            let enemy_center_x = enemy.x + enemy_width / 2;
                            let enemy_center_y = enemy.y + enemy_height / 2;
                            let death_particles =
                                create_explosion_particles(enemy_center_x, enemy_center_y);
                            self.particles.extend(death_particles);

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
                // Create particles at collision point
                let enemy_center_x = enemy.x + enemy_width / 2;
                let enemy_center_y = enemy.y + enemy_height / 2;
                let collision_particles =
                    create_explosion_particles(enemy_center_x, enemy_center_y);
                self.particles.extend(collision_particles);

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
