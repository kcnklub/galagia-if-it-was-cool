mod entities;

use color_eyre::Result;
use crossterm::{
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
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

#[derive(Debug, Default)]
struct KeyState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    fire: bool,
}

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
    /// Currently pressed keys
    keys: KeyState,
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
            keys: KeyState::default(),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;

            // Update game state
            if self.game_state == GameState::Playing {
                self.update_game();
            }
        }
        Ok(())
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

    /// Reads the crossterm events and updates the state of [`App`].
    fn handle_crossterm_events(&mut self) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .unwrap();

        // Process all available events without blocking for responsive input
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => {
                    writeln!(file, "EVENT RECEIVED: {:?} kind: {:?}", key.code, key.kind).unwrap();
                    self.on_key_event(key);
                }
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        // Process movement based on currently held keys this frame
        if self.game_state == GameState::Playing {
            self.process_held_keys();
        }

        // Small sleep to maintain ~60 FPS and prevent CPU spinning
        std::thread::sleep(Duration::from_millis(16));

        Ok(())
    }

    /// Handles key events (both press and release)
    fn on_key_event(&mut self, key: KeyEvent) {
        match self.game_state {
            GameState::Playing => self.handle_playing_keys(key),
            GameState::Paused => self.handle_paused_keys(key),
            GameState::GameOver => self.handle_game_over_keys(key),
        }
    }

    /// Process movement based on currently held keys
    fn process_held_keys(&mut self) {
        let player_width = self.player.get_width();
        let player_height = self.player.get_height();

        // Handle movement
        if self.keys.left && self.player.x > 0 {
            self.player.move_left(0);
        }
        if self.keys.right && self.player.x < self.screen_width.saturating_sub(player_width) {
            self.player
                .move_right(self.screen_width.saturating_sub(player_width));
        }
        if self.keys.up && self.player.y > 2 {
            self.player.move_up(2);
        }
        if self.keys.down && self.player.y < self.screen_height.saturating_sub(player_height + 1) {
            self.player
                .move_down(self.screen_height.saturating_sub(player_height + 1));
        }

        // Handle firing
        if self.keys.fire && self.player.can_fire() {
            let fire_x = self.player.x + player_width / 2;
            self.projectiles.push(Projectile::new(
                fire_x,
                self.player.y.saturating_sub(1),
                ProjectileOwner::Player,
            ));
            self.player.reset_cooldown();
        }
    }

    fn handle_playing_keys(&mut self, key: KeyEvent) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
            .unwrap();

        writeln!(file, "Key event: {:?} kind: {:?}", key.code, key.kind).unwrap();

        match key.kind {
            KeyEventKind::Press => {
                match (key.modifiers, key.code) {
                    // Quit
                    (_, KeyCode::Char('q') | KeyCode::Esc)
                    | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                        self.quit()
                    }

                    // Pause
                    (_, KeyCode::Char('p') | KeyCode::Char('P')) => {
                        self.game_state = GameState::Paused;
                    }

                    // Movement - Left
                    (_, KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A')) => {
                        self.keys.left = true;
                        self.keys.right = false; // Clear opposite direction
                        writeln!(
                            file,
                            "Press LEFT - keys: left={} right={} up={} down={}",
                            self.keys.left, self.keys.right, self.keys.up, self.keys.down
                        )
                        .unwrap();
                    }

                    // Movement - Right
                    (_, KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D')) => {
                        self.keys.left = false; // Clear opposite direction
                        self.keys.right = true;
                        writeln!(
                            file,
                            "Press RIGHT - keys: left={} right={} up={} down={}",
                            self.keys.left, self.keys.right, self.keys.up, self.keys.down
                        )
                        .unwrap();
                    }

                    // Movement - Up
                    (_, KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W')) => {
                        self.keys.down = false; // Clear opposite direction
                        self.keys.up = true;
                        writeln!(
                            file,
                            "Press UP - keys: left={} right={} up={} down={}",
                            self.keys.left, self.keys.right, self.keys.up, self.keys.down
                        )
                        .unwrap();
                    }

                    // Movement - Down
                    (_, KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S')) => {
                        self.keys.up = false; // Clear opposite direction
                        self.keys.down = true;
                        writeln!(
                            file,
                            "Press DOWN - keys: left={} right={} up={} down={}",
                            self.keys.left, self.keys.right, self.keys.up, self.keys.down
                        )
                        .unwrap();
                    }

                    // Fire
                    (_, KeyCode::Char(' ')) => {
                        self.keys.fire = true;
                    }

                    _ => {}
                }
            }
            KeyEventKind::Release => {
                writeln!(
                    file,
                    "Before release - left:{} right:{} up:{} down:{}",
                    self.keys.left, self.keys.right, self.keys.up, self.keys.down
                )
                .unwrap();

                match key.code {
                    // Movement - Left
                    KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                        self.keys.left = false;
                    }

                    // Movement - Right
                    KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                        self.keys.right = false;
                    }

                    // Movement - Up
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                        self.keys.up = false;
                    }

                    // Movement - Down
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                        self.keys.down = false;
                    }

                    // Fire
                    KeyCode::Char(' ') => {
                        self.keys.fire = false;
                    }

                    _ => {}
                }

                writeln!(
                    file,
                    "After release - left:{} right:{} up:{} down:{}",
                    self.keys.left, self.keys.right, self.keys.up, self.keys.down
                )
                .unwrap();
            }
            _ => {}
        }
    }

    fn handle_paused_keys(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            // Quit
            (_, KeyCode::Char('q') | KeyCode::Esc)
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),

            // Unpause
            (_, KeyCode::Char('p') | KeyCode::Char('P')) => {
                self.game_state = GameState::Playing;
            }

            _ => {}
        }
    }

    fn handle_game_over_keys(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            // Quit
            (_, KeyCode::Char('q') | KeyCode::Esc)
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),

            // Restart
            (_, KeyCode::Char('r') | KeyCode::Char('R')) => {
                *self = Self::new();
            }

            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
