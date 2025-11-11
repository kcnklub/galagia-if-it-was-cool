use crate::entities::{
    Enemy, EnemyType, GameState, Particle, Pickup, Player, Projectile, ProjectileOwner,
    ProjectileType,
};
use rand::Rng;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// View struct that holds all game state needed for rendering
pub struct RenderView<'a> {
    pub game_state: GameState,
    pub player: &'a Player,
    pub enemies: &'a [Enemy],
    pub projectiles: &'a [Projectile],
    pub particles: &'a [Particle],
    pub pickups: &'a [Pickup],
    pub score: u32,
    pub frame_count: u64,
    pub area: Rect,
    pub edge_width: u16,
    pub fps: u32,
    pub elapsed_time_secs: u64,
}

/// Handles all rendering responsibilities for the game
pub struct GameRenderer {
    // Future: could add theme/config fields here
}

impl GameRenderer {
    /// Creates a new GameRenderer
    pub fn new() -> Self {
        Self {}
    }

    /// Main render method that dispatches to state-specific renderers
    pub fn render(&self, frame: &mut Frame, view: &RenderView) {
        match view.game_state {
            GameState::Playing => self.render_game(frame, view),
            GameState::Paused => self.render_paused(frame, view),
            GameState::GameOver => self.render_game_over(frame, view),
        }
    }

    /// Renders the active gameplay screen
    fn render_game(&self, frame: &mut Frame, view: &RenderView) {
        let area = view.area;

        // Create a narrower centered game area with borders
        let game_area = if view.edge_width > 0 {
            // Calculate the narrowed area (subtract edge_width from each side)
            let total_margin = view.edge_width * 2;
            let game_width = area.width.saturating_sub(total_margin);
            let centered_area = Rect {
                x: area.x + view.edge_width,
                y: area.y,
                width: game_width,
                height: area.height,
            };

            // Render block with borders around the narrowed area
            let block = Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(Color::DarkGray));
            let inner = block.inner(centered_area);
            frame.render_widget(block, centered_area);
            inner
        } else {
            area
        };

        // Render stars (simple background)
        if view.frame_count % 10 < 5 {
            let star_text = (0..game_area.height)
                .map(|_| {
                    let mut rng = rand::rng();
                    if rng.random_bool(0.05) { "." } else { " " }
                })
                .collect::<Vec<_>>()
                .join("\n");
            frame.render_widget(
                Paragraph::new(star_text).style(Style::default().fg(Color::DarkGray)),
                game_area,
            );
        }

        // Render player - optimized with batched multi-line rendering
        if view.player.is_alive() {
            let sprite_lines = view.player.get_sprite_lines();
            let player_width = view.player.get_width();
            // Flash white when taking damage, otherwise green
            let player_color = if view.player.is_flashing() {
                Color::White
            } else {
                Color::Green
            };

            // Build multi-line text with consistent styling
            let text: Vec<Line> = sprite_lines
                .iter()
                .map(|line| {
                    Line::from(*line).style(
                        Style::default()
                            .fg(player_color)
                            .add_modifier(Modifier::BOLD),
                    )
                })
                .collect();

            let player_area = Rect {
                x: game_area.x + view.player.x,
                y: game_area.y + view.player.y,
                width: player_width,
                height: sprite_lines.len() as u16,
            };

            // Single widget call for entire player sprite
            if view.player.y + sprite_lines.len() as u16 <= game_area.height
                && view.player.x + player_width < game_area.width
            {
                frame.render_widget(Paragraph::new(text), player_area);
            }
        }

        // Render enemies - optimized with batched multi-line rendering
        for enemy in view.enemies {
            let sprite_lines = enemy.get_sprite_lines();
            let enemy_width = enemy.get_width();
            // Flash white when taking damage, otherwise use normal color
            let color = if enemy.is_flashing() {
                Color::White
            } else {
                match enemy.enemy_type {
                    EnemyType::Basic => Color::Red,
                    EnemyType::Fast => Color::Magenta,
                    EnemyType::Tank => Color::Yellow,
                }
            };

            // Build multi-line text with consistent styling
            let text: Vec<Line> = sprite_lines
                .iter()
                .map(|line| {
                    Line::from(*line).style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                })
                .collect();

            let enemy_area = Rect {
                x: game_area.x + enemy.x,
                y: game_area.y + enemy.y,
                width: enemy_width,
                height: sprite_lines.len() as u16,
            };

            // Single widget call for entire enemy sprite
            if enemy.y + sprite_lines.len() as u16 <= game_area.height
                && enemy.x + enemy_width < game_area.width
            {
                frame.render_widget(Paragraph::new(text), enemy_area);
            }
        }

        // Render projectiles - optimized with direct buffer access
        let buffer = frame.buffer_mut();
        for projectile in view.projectiles {
            if projectile.x < game_area.width && projectile.y < game_area.height {
                let (char, color) = match (&projectile.projectile_type, &projectile.owner) {
                    (ProjectileType::Bullet, ProjectileOwner::Player) => ('|', Color::Yellow),
                    (ProjectileType::Slash, ProjectileOwner::Player) => ('~', Color::Cyan),
                    (ProjectileType::BugShot, ProjectileOwner::Player) => ('•', Color::Green),
                    (ProjectileType::BomberProjectile, ProjectileOwner::Player) => {
                        // Blinking effect when near explosion
                        if projectile.lifetime.unwrap_or(1) <= 10 {
                            ('O', Color::Red)
                        } else {
                            ('O', Color::LightRed)
                        }
                    }
                    (_, ProjectileOwner::Enemy) => ('!', Color::Magenta),
                };

                buffer.set_string(
                    game_area.x + projectile.x,
                    game_area.y + projectile.y,
                    char.to_string(),
                    Style::default().fg(color),
                );
            }
        }

        // Render particles - optimized with direct buffer access
        for particle in view.particles {
            if particle.x < game_area.width && particle.y < game_area.height {
                // Color particles based on their lifetime (fade effect)
                let color = if particle.lifetime > 8 {
                    Color::Red
                } else if particle.lifetime > 4 {
                    Color::LightRed
                } else {
                    Color::Yellow
                };

                buffer.set_string(
                    game_area.x + particle.x,
                    game_area.y + particle.y,
                    particle.char.to_string(),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                );
            }
        }

        // Render pickups - optimized with direct buffer access
        for pickup in view.pickups {
            if pickup.x < game_area.width && pickup.y < game_area.height {
                buffer.set_string(
                    game_area.x + pickup.x,
                    game_area.y + pickup.y,
                    pickup.get_char().to_string(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                );
            }
        }

        // Stats overlay at the top - left side
        let stats_left = Line::from(vec![
            Span::styled("Score: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", view.score),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  HP: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}%", view.player.health),
                if view.player.health > 50 {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if view.player.health > 25 {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                },
            ),
            Span::styled("  Enemies: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", view.enemies.len()),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Weapon: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                view.player.current_weapon.get_name(),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  FPS: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", view.fps),
                Style::default()
                    .fg(Color::White)
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

        // Timer in center of header
        let minutes = view.elapsed_time_secs / 60;
        let seconds = view.elapsed_time_secs % 60;
        let timer_text = Line::from(vec![
            Span::styled("Time: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:02}:{:02}", minutes, seconds),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let timer_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };

        frame.render_widget(Paragraph::new(timer_text).centered(), timer_area);

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

    /// Renders the pause screen with overlay
    fn render_paused(&self, frame: &mut Frame, view: &RenderView) {
        // First render the game screen
        self.render_game(frame, view);

        let area = view.area;
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

    /// Renders the game over screen
    fn render_game_over(&self, frame: &mut Frame, view: &RenderView) {
        let area = view.area;
        let minutes = view.elapsed_time_secs / 60;
        let seconds = view.elapsed_time_secs % 60;

        let game_over_text = vec![
            Line::from(""),
            Line::from("╔═══════════════════════════╗").centered().red(),
            Line::from("║      GAME OVER!           ║")
                .centered()
                .red()
                .bold(),
            Line::from("╚═══════════════════════════╝").centered().red(),
            Line::from(""),
            Line::from(format!("Final Score: {}", view.score))
                .centered()
                .yellow()
                .bold(),
            Line::from(format!("Time Survived: {:02}:{:02}", minutes, seconds))
                .centered()
                .cyan()
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
