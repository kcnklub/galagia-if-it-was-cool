mod entities;
mod input;

use color_eyre::Result;
use crossterm::{
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::Rng;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::io::Write;
use std::time::Duration;
use std::{fs::OpenOptions, io::stdout};

use entities::{Enemy, EnemyType, GameState, Player, Projectile, ProjectileOwner};
use input::{InputAction, InputManager};

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
    /// Player score
    score: u32,
    /// Frame counter for timing
    frame_count: u64,
    /// Last known screen dimensions
    screen_width: u16,
    screen_height: u16,
    /// Input manager
    input_manager: InputManager,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        // Start with reasonable defaults, will be updated on first render
        let screen_width = 120;
        let screen_height = 30;

        Self {
            running: true,
            game_state: GameState::Playing,
            player: Player::new(screen_width / 2, screen_height - 5),
            enemies: Vec::new(),
            projectiles: Vec::new(),
            score: 0,
            frame_count: 0,
            screen_width,
            screen_height,
            input_manager: InputManager::new(),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;

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
            std::thread::sleep(Duration::from_millis(16));
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
                    let min_x = 0;
                    self.player.move_left(min_x);
                }
                InputAction::MoveRight => {
                    let max_x = self.screen_width.saturating_sub(self.player.get_width());
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
                    if let Some(projectile) = self.player.try_fire() {
                        self.projectiles.push(projectile);
                    }
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

        // Remove out-of-bounds projectiles
        self.projectiles
            .retain(|p| !p.is_out_of_bounds(self.screen_height));

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

        // Spawn with enough space for the enemy sprite
        let x = rng.gen_range(1..self.screen_width.saturating_sub(enemy_width + 1));

        self.enemies.push(Enemy::new(x, 2, enemy_type));
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
    }

    /// Renders the user interface.
    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Update screen dimensions based on actual terminal size
        self.screen_width = area.width;
        self.screen_height = area.height;

        match self.game_state {
            GameState::Playing => self.render_game(frame, area),
            GameState::Paused => self.render_paused(frame, area),
            GameState::GameOver => self.render_game_over(frame, area),
        }
    }

    fn render_game(&self, frame: &mut Frame, area: Rect) {
        // Use the entire area for the game
        let inner = area;

        // Render stars (simple background)
        if self.frame_count % 10 < 5 {
            let star_text = (0..inner.height)
                .map(|_| {
                    let mut rng = rand::thread_rng();
                    if rng.gen_bool(0.05) { "." } else { " " }
                })
                .collect::<Vec<_>>()
                .join("\n");
            frame.render_widget(
                Paragraph::new(star_text).style(Style::default().fg(Color::DarkGray)),
                inner,
            );
        }

        // Render player
        if self.player.is_alive() {
            let sprite_lines = self.player.get_sprite_lines();
            let player_width = self.player.get_width();

            for (i, line) in sprite_lines.iter().enumerate() {
                let y_pos = self.player.y + i as u16;
                if y_pos < inner.height && self.player.x + player_width < inner.width {
                    let player_area = Rect {
                        x: inner.x + self.player.x,
                        y: inner.y + y_pos,
                        width: player_width,
                        height: 1,
                    };
                    frame.render_widget(
                        Paragraph::new(*line).style(
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                        player_area,
                    );
                }
            }
        }

        // Render enemies
        for enemy in &self.enemies {
            let sprite_lines = enemy.get_sprite_lines();
            let enemy_width = enemy.get_width();
            let color = match enemy.enemy_type {
                EnemyType::Basic => Color::Red,
                EnemyType::Fast => Color::Magenta,
                EnemyType::Tank => Color::Yellow,
            };

            for (i, line) in sprite_lines.iter().enumerate() {
                let y_pos = enemy.y + i as u16;
                if y_pos < inner.height && enemy.x + enemy_width < inner.width {
                    let enemy_area = Rect {
                        x: inner.x + enemy.x,
                        y: inner.y + y_pos,
                        width: enemy_width,
                        height: 1,
                    };
                    frame.render_widget(
                        Paragraph::new(*line)
                            .style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                        enemy_area,
                    );
                }
            }
        }

        // Render projectiles
        for projectile in &self.projectiles {
            if projectile.x < inner.width && projectile.y < inner.height {
                let proj_area = Rect {
                    x: inner.x + projectile.x,
                    y: inner.y + projectile.y,
                    width: 1,
                    height: 1,
                };
                let (char, color) = match projectile.owner {
                    ProjectileOwner::Player => ('|', Color::Yellow),
                    ProjectileOwner::Enemy => ('!', Color::Magenta),
                };
                frame.render_widget(
                    Paragraph::new(char.to_string()).style(Style::default().fg(color)),
                    proj_area,
                );
            }
        }

        // Stats overlay at the top
        let stats_left = Line::from(vec![
            Span::styled("Score: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", self.score),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  HP: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}%", self.player.health),
                if self.player.health > 50 {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if self.player.health > 25 {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                },
            ),
            Span::styled("  Enemies: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", self.enemies.len()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let stats_area = Rect {
            x: area.x + 1,
            y: area.y,
            width: area.width.saturating_sub(2),
            height: 1,
        };

        frame.render_widget(Paragraph::new(stats_left), stats_area);

        // Controls hint at bottom
        let controls = Line::from(vec![Span::styled(
            "[WASD/Arrows: Move] [Space: Fire] [P: Pause] [Q: Quit]",
            Style::default().fg(Color::DarkGray),
        )]);

        let controls_area = Rect {
            x: area.x + 1,
            y: area.y + area.height.saturating_sub(1),
            width: area.width.saturating_sub(2),
            height: 1,
        };

        frame.render_widget(Paragraph::new(controls).centered(), controls_area);
    }

    fn render_paused(&self, frame: &mut Frame, area: Rect) {
        self.render_game(frame, area);

        let pause_text = vec![
            Line::from(""),
            Line::from("PAUSED").centered().bold().yellow(),
            Line::from(""),
            Line::from("Press P to resume").centered().white(),
        ];

        let pause_area = Rect {
            x: area.width / 2 - 15,
            y: area.height / 2 - 3,
            width: 30,
            height: 6,
        };

        frame.render_widget(
            Paragraph::new(pause_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .alignment(Alignment::Center),
            pause_area,
        );
    }

    fn render_game_over(&self, frame: &mut Frame, area: Rect) {
        let game_over_text = vec![
            Line::from(""),
            Line::from("╔═══════════════════════════╗").centered().red(),
            Line::from("║      GAME OVER!           ║")
                .centered()
                .red()
                .bold(),
            Line::from("╚═══════════════════════════╝").centered().red(),
            Line::from(""),
            Line::from(format!("Final Score: {}", self.score))
                .centered()
                .yellow()
                .bold(),
            Line::from(""),
            Line::from("Press R to restart").centered().white(),
            Line::from("Press Q to quit").centered().white(),
        ];

        frame.render_widget(
            Paragraph::new(game_over_text)
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center),
            area,
        );
    }
}
