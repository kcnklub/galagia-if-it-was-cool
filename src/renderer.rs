use crate::entities::{Enemy, EnemyType, GameState, Pickup, Player, Projectile, ProjectileOwner, ProjectileType};
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
    pub pickups: &'a [Pickup],
    pub score: u32,
    pub frame_count: u64,
    pub area: Rect,
    pub edge_width: u16,
    pub fps: u32,
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
                    let mut rng = rand::thread_rng();
                    if rng.gen_bool(0.05) {
                        "."
                    } else {
                        " "
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            frame.render_widget(
                Paragraph::new(star_text).style(Style::default().fg(Color::DarkGray)),
                game_area,
            );
        }

        // Render player
        if view.player.is_alive() {
            let sprite_lines = view.player.get_sprite_lines();
            let player_width = view.player.get_width();

            for (i, line) in sprite_lines.iter().enumerate() {
                let y_pos = view.player.y + i as u16;
                if y_pos < game_area.height && view.player.x + player_width < game_area.width {
                    let player_area = Rect {
                        x: game_area.x + view.player.x,
                        y: game_area.y + y_pos,
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
        for enemy in view.enemies {
            let sprite_lines = enemy.get_sprite_lines();
            let enemy_width = enemy.get_width();
            let color = match enemy.enemy_type {
                EnemyType::Basic => Color::Red,
                EnemyType::Fast => Color::Magenta,
                EnemyType::Tank => Color::Yellow,
            };

            for (i, line) in sprite_lines.iter().enumerate() {
                let y_pos = enemy.y + i as u16;
                if y_pos < game_area.height && enemy.x + enemy_width < game_area.width {
                    let enemy_area = Rect {
                        x: game_area.x + enemy.x,
                        y: game_area.y + y_pos,
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
        for projectile in view.projectiles {
            if projectile.x < game_area.width && projectile.y < game_area.height {
                let proj_area = Rect {
                    x: game_area.x + projectile.x,
                    y: game_area.y + projectile.y,
                    width: 1,
                    height: 1,
                };
                let (char, color) = match (&projectile.projectile_type, &projectile.owner) {
                    (ProjectileType::Bullet, ProjectileOwner::Player) => ('|', Color::Yellow),
                    (ProjectileType::Slash, ProjectileOwner::Player) => ('~', Color::Cyan),
                    (ProjectileType::BugShot, ProjectileOwner::Player) => ('•', Color::Green),
                    (_, ProjectileOwner::Enemy) => ('!', Color::Magenta),
                };
                frame.render_widget(
                    Paragraph::new(char.to_string()).style(Style::default().fg(color)),
                    proj_area,
                );
            }
        }

        // Render pickups
        for pickup in view.pickups {
            if pickup.x < game_area.width && pickup.y < game_area.height {
                let pickup_area = Rect {
                    x: game_area.x + pickup.x,
                    y: game_area.y + pickup.y,
                    width: 1,
                    height: 1,
                };
                frame.render_widget(
                    Paragraph::new(pickup.get_char().to_string())
                        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    pickup_area,
                );
            }
        }

        // Stats overlay at the top
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
