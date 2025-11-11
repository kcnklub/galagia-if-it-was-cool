use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

use crate::entities::GameState;

/// Represents semantic game actions that can be triggered by input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Fire,
    Pause,
    Resume,
    Restart,
    Quit,
}

/// Tracks the state of keys that can be held down for continuous input
#[derive(Debug, Default)]
struct KeyState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    fire: bool,
}

/// Manages input polling and translates raw key events into game actions
pub struct InputManager {
    key_state: KeyState,
    oneshot_actions: Vec<InputAction>,
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputManager {
    /// Creates a new InputManager with default key state
    pub fn new() -> Self {
        Self {
            key_state: KeyState::default(),
            oneshot_actions: Vec::new(),
        }
    }

    /// Polls for all input events and stores one-shot actions
    /// Should be called once per frame before getting actions
    pub fn poll_events(&mut self, game_state: &GameState) -> color_eyre::Result<()> {
        // Clear previous one-shot actions
        self.oneshot_actions.clear();

        // Poll for all available events without blocking
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key_event) => {
                    self.handle_key_event(key_event, game_state);
                }
                Event::Mouse(_) => {
                    // Mouse events currently ignored
                }
                Event::Resize(_, _) => {
                    // Resize events handled elsewhere
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Processes a key event and updates key state and one-shot actions
    fn handle_key_event(&mut self, key_event: KeyEvent, game_state: &GameState) {
        match key_event.kind {
            KeyEventKind::Press => {
                self.handle_key_press(key_event, game_state);
            }
            KeyEventKind::Release => {
                self.handle_key_release(key_event.code);
            }
            _ => {}
        }
    }

    /// Handles key press events
    fn handle_key_press(&mut self, key_event: KeyEvent, game_state: &GameState) {
        // Check for quit keys first (works in any state)
        if matches!(
            key_event.code,
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc
        ) || (key_event.code == KeyCode::Char('c')
            && key_event.modifiers.contains(KeyModifiers::CONTROL))
        {
            self.oneshot_actions.push(InputAction::Quit);
            return;
        }

        // State-specific one-shot actions
        match game_state {
            GameState::Playing => {
                if matches!(key_event.code, KeyCode::Char('p') | KeyCode::Char('P')) {
                    self.oneshot_actions.push(InputAction::Pause);
                    return;
                }
            }
            GameState::Paused => {
                if matches!(key_event.code, KeyCode::Char('p') | KeyCode::Char('P')) {
                    self.oneshot_actions.push(InputAction::Resume);
                    return;
                }
            }
            GameState::GameOver => {
                if matches!(key_event.code, KeyCode::Char('r') | KeyCode::Char('R')) {
                    self.oneshot_actions.push(InputAction::Restart);
                    return;
                }
            }
        }

        // Continuous action keys (only tracked in Playing state)
        if *game_state == GameState::Playing {
            match key_event.code {
                // Movement keys - WASD
                KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                    self.key_state.up = true;
                    self.key_state.down = false;
                }
                KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                    self.key_state.down = true;
                    self.key_state.up = false;
                }
                KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => {
                    self.key_state.left = true;
                    self.key_state.right = false;
                }
                KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => {
                    self.key_state.right = true;
                    self.key_state.left = false;
                }
                // Fire key
                KeyCode::Char(' ') => {
                    self.key_state.fire = true;
                }
                _ => {}
            }
        }
    }

    /// Handles key release events
    fn handle_key_release(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                self.key_state.up = false;
            }
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                self.key_state.down = false;
            }
            KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => {
                self.key_state.left = false;
            }
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => {
                self.key_state.right = false;
            }
            KeyCode::Char(' ') => {
                self.key_state.fire = false;
            }
            _ => {}
        }
    }

    /// Returns all actions for this frame (both continuous and one-shot)
    /// Must be called after poll_events()
    pub fn get_actions(&self, game_state: &GameState) -> Vec<InputAction> {
        let mut actions = Vec::new();

        // Add one-shot actions first
        actions.extend_from_slice(&self.oneshot_actions);

        // Add continuous actions based on held keys (only in Playing state)
        if *game_state == GameState::Playing {
            if self.key_state.left {
                actions.push(InputAction::MoveLeft);
            }
            if self.key_state.right {
                actions.push(InputAction::MoveRight);
            }
            if self.key_state.up {
                actions.push(InputAction::MoveUp);
            }
            if self.key_state.down {
                actions.push(InputAction::MoveDown);
            }
            if self.key_state.fire {
                actions.push(InputAction::Fire);
            }
        }

        actions
    }
}
