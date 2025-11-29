#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyType {
    Basic,
    Fast,
    Tank,
    Sniper,
    Spinner,
    Charger,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementState {
    Descending,
    ZigzagLeft,
    ZigzagRight,
    Strafing,
    Circling,
    Charging,
    Paused,
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub x: u16,
    pub y: u16,
    pub health: u8,
    pub enemy_type: EnemyType,
    pub fire_cooldown: u8,
    /// Optional formation this enemy belongs to
    pub formation_id: Option<usize>,
    /// Offset from formation center
    pub formation_offset: (i16, i16),
    pub damage_flash_frames: u8,
    /// Current movement state for individual behavior
    pub movement_state: MovementState,
    /// Timer for movement pattern timing
    pub movement_timer: u8,
    /// Target x position for targeted movement
    pub target_x: Option<u16>,
}

impl Enemy {
    pub fn new_in_formation(
        x: u16,
        y: u16,
        enemy_type: EnemyType,
        formation_id: usize,
        offset: (i16, i16),
    ) -> Self {
        let health = match enemy_type {
            EnemyType::Basic => 15,
            EnemyType::Fast => 10,
            EnemyType::Tank => 30,
            EnemyType::Sniper => 8,
            EnemyType::Spinner => 18,
            EnemyType::Charger => 25,
        };

        Self {
            x,
            y,
            health,
            enemy_type,
            fire_cooldown: 0,
            formation_id: Some(formation_id),
            formation_offset: offset,
            damage_flash_frames: 0,
            movement_state: MovementState::Descending,
            movement_timer: 0,
            target_x: None,
        }
    }

    pub fn update(&mut self) {
        // Update damage flash
        if self.damage_flash_frames > 0 {
            self.damage_flash_frames -= 1;
        }

        // Enemies in formations don't move on their own - they follow the formation
        if self.formation_id.is_some() {
            self.fire_cooldown = self.fire_cooldown.wrapping_add(1);
            return;
        }

        // Update movement timer
        self.movement_timer = self.movement_timer.wrapping_add(1);

        // Execute movement pattern based on enemy type
        match self.enemy_type {
            EnemyType::Basic => self.update_basic_movement(),
            EnemyType::Fast => self.update_fast_movement(),
            EnemyType::Tank => self.update_tank_movement(),
            EnemyType::Sniper => self.update_sniper_movement(),
            EnemyType::Spinner => self.update_spinner_movement(),
            EnemyType::Charger => self.update_charger_movement(),
        }

        self.fire_cooldown = self.fire_cooldown.wrapping_add(1);
    }

    fn update_basic_movement(&mut self) {
        // Simple downward movement
        if self.movement_timer.is_multiple_of(8) {
            self.y += 1;
        }
    }

    fn update_fast_movement(&mut self) {
        // Zigzag pattern while descending
        if self.movement_timer.is_multiple_of(5) {
            self.y += 1;
            
            // Change horizontal direction every 20 frames
            match self.movement_state {
                MovementState::ZigzagLeft => {
                    if self.x > 5 { self.x -= 1; }
                    if self.movement_timer.is_multiple_of(20) {
                        self.movement_state = MovementState::ZigzagRight;
                    }
                }
                MovementState::ZigzagRight => {
                    if self.x < 75 { self.x += 1; }
                    if self.movement_timer.is_multiple_of(20) {
                        self.movement_state = MovementState::ZigzagLeft;
                    }
                }
                _ => self.movement_state = MovementState::ZigzagLeft,
            }
        }
    }

    fn update_tank_movement(&mut self) {
        // Slow steady descent with occasional pauses
        match self.movement_state {
            MovementState::Paused => {
                if self.movement_timer.is_multiple_of(30) {
                    self.movement_state = MovementState::Descending;
                }
            }
            MovementState::Descending => {
                if self.movement_timer.is_multiple_of(10) {
                    self.y += 1;
                }
                // Pause every 40 frames
                if self.movement_timer.is_multiple_of(40) {
                    self.movement_state = MovementState::Paused;
                }
            }
            _ => self.movement_state = MovementState::Descending,
        }
    }

    fn update_sniper_movement(&mut self) {
        // Maintains distance, strafes horizontally
        if self.movement_timer.is_multiple_of(9) {
            self.y += 1;
        }
        
        // Horizontal strafing
        if self.movement_timer.is_multiple_of(6) {
            match self.movement_state {
                MovementState::Strafing => {
                    if self.target_x.is_none() {
                        // Pick a random target x position
                        self.target_x = Some(20 + (self.movement_timer % 40) as u16);
                    }
                    
                    if let Some(target) = self.target_x {
                        if self.x < target {
                            self.x += 1;
                        } else if self.x > target {
                            self.x -= 1;
                        } else {
                            // Reached target, pick new one
                            self.target_x = Some(20 + (self.movement_timer % 40) as u16);
                        }
                    }
                }
                _ => self.movement_state = MovementState::Strafing,
            }
        }
    }

    fn update_spinner_movement(&mut self) {
        // Circular/spiral movement pattern
        if self.movement_timer.is_multiple_of(8) {
            self.y += 1;
        }
        
        // Circular motion
        let angle = (self.movement_timer as f32 * 0.1) % (2.0 * std::f32::consts::PI);
        let radius = 3.0;
        let dx = (angle.cos() * radius) as i16;
        
        if self.movement_timer.is_multiple_of(3) {
            let new_x = self.x as i16 + dx;
            if (5..=75).contains(&new_x) {
                self.x = new_x as u16;
            }
        }
    }

    fn update_charger_movement(&mut self) {
        // Brief acceleration toward player when aligned
        if self.movement_timer.is_multiple_of(6) {
            self.y += 1;
        }
        
        // Check if aligned with player (simplified - would need player position)
        // For now, charge periodically
        if self.movement_timer.is_multiple_of(60) {
            self.movement_state = MovementState::Charging;
        }
        
        if self.movement_state == MovementState::Charging {
            // Rapid descent for 10 frames
            if !self.movement_timer.is_multiple_of(10) {
                self.y += 2; // Double speed during charge
            } else {
                self.movement_state = MovementState::Descending;
            }
        }
    }

    /// Update position based on formation center
    pub fn update_formation_position(&mut self, center_x: u16, center_y: u16) {
        let new_x = center_x as i16 + self.formation_offset.0;
        let new_y = center_y as i16 + self.formation_offset.1;

        if new_x >= 0 {
            self.x = new_x as u16;
        }
        if new_y >= 0 {
            self.y = new_y as u16;
        }
    }

    pub fn can_fire(&self) -> bool {
        self.fire_cooldown.is_multiple_of(120)  // Increased from 30 to 120 (2 seconds at 60 FPS)
    }

    pub fn take_damage(&mut self, damage: u8) {
        self.health = self.health.saturating_sub(damage);
        // Set flash timer to 10 frames (about 1/6 second at 60 FPS)
        self.damage_flash_frames = 10;
    }

    pub fn is_flashing(&self) -> bool {
        self.damage_flash_frames > 0
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn get_sprite_lines(&self) -> Vec<&'static str> {
        match self.enemy_type {
            EnemyType::Basic => vec!["  \\|/  ", " {===} ", "  /_\\  "],
            EnemyType::Fast => vec!["  <*>  ", " <|||> ", "  <*>  "],
            EnemyType::Tank => vec![" [===] ", " |###| ", " [===] "],
            EnemyType::Sniper => vec!["  ^|^  ", "  |o|  ", "  '|'  "],
            EnemyType::Spinner => vec!["  (@)  ", " /@|@\\ ", "  (@)  "],
            EnemyType::Charger => vec![" \\ V / ", "  \\V/  ", "   V   "],
        }
    }

    pub fn get_width(&self) -> u16 {
        match self.enemy_type {
            EnemyType::Basic => 7,
            EnemyType::Fast => 8,  // Sprite size for dark-fighter
            EnemyType::Tank => 8,  // Sprite size for dark-tanker
            EnemyType::Sniper => 7,
            EnemyType::Spinner => 7,
            EnemyType::Charger => 7,
        }
    }

    pub fn get_height(&self) -> u16 {
        match self.enemy_type {
            EnemyType::Basic => 3,
            EnemyType::Fast => 5,  // Sprite size for dark-fighter
            EnemyType::Tank => 5,  // Sprite size for dark-tanker
            EnemyType::Sniper => 3,
            EnemyType::Spinner => 3,
            EnemyType::Charger => 3,
        }
    }

    pub fn get_points(&self) -> u32 {
        match self.enemy_type {
            EnemyType::Basic => 10,
            EnemyType::Fast => 20,
            EnemyType::Tank => 30,
            EnemyType::Sniper => 40,
            EnemyType::Spinner => 50,
            EnemyType::Charger => 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enemy_health_by_type() {
        let basic = Enemy::new_in_formation(10, 10, EnemyType::Basic, 0, (0, 0));
        assert_eq!(basic.health, 15);

        let fast = Enemy::new_in_formation(10, 10, EnemyType::Fast, 0, (0, 0));
        assert_eq!(fast.health, 10);

        let tank = Enemy::new_in_formation(10, 10, EnemyType::Tank, 0, (0, 0));
        assert_eq!(tank.health, 30);
    }

    #[test]
    fn test_enemy_points_by_type() {
        let basic = Enemy::new_in_formation(10, 10, EnemyType::Basic, 0, (0, 0));
        assert_eq!(basic.get_points(), 10);

        let fast = Enemy::new_in_formation(10, 10, EnemyType::Fast, 0, (0, 0));
        assert_eq!(fast.get_points(), 20);

        let tank = Enemy::new_in_formation(10, 10, EnemyType::Tank, 0, (0, 0));
        assert_eq!(tank.get_points(), 30);

        let sniper = Enemy::new_in_formation(10, 10, EnemyType::Sniper, 0, (0, 0));
        assert_eq!(sniper.get_points(), 40);

        let spinner = Enemy::new_in_formation(10, 10, EnemyType::Spinner, 0, (0, 0));
        assert_eq!(spinner.get_points(), 50);

        let charger = Enemy::new_in_formation(10, 10, EnemyType::Charger, 0, (0, 0));
        assert_eq!(charger.get_points(), 60);
    }

    #[test]
    fn test_new_enemy_types_health() {
        let sniper = Enemy::new_in_formation(10, 10, EnemyType::Sniper, 0, (0, 0));
        assert_eq!(sniper.health, 8);

        let spinner = Enemy::new_in_formation(10, 10, EnemyType::Spinner, 0, (0, 0));
        assert_eq!(spinner.health, 18);

        let charger = Enemy::new_in_formation(10, 10, EnemyType::Charger, 0, (0, 0));
        assert_eq!(charger.health, 25);
    }

    #[test]
    fn test_enemy_take_damage() {
        let mut enemy = Enemy::new_in_formation(10, 10, EnemyType::Basic, 0, (0, 0));
        enemy.take_damage(5);
        assert_eq!(enemy.health, 10);
        assert!(enemy.is_alive());

        enemy.take_damage(10);
        assert_eq!(enemy.health, 0);
        assert!(!enemy.is_alive());
    }

    #[test]
    fn test_enemy_update_formation_position() {
        let mut enemy = Enemy::new_in_formation(10, 10, EnemyType::Basic, 0, (5, 3));
        enemy.update_formation_position(20, 15);
        assert_eq!(enemy.x, 25);
        assert_eq!(enemy.y, 18);
    }

    #[test]
    fn test_enemy_update_formation_position_negative_offset() {
        let mut enemy = Enemy::new_in_formation(10, 10, EnemyType::Basic, 0, (-8, -2));
        enemy.update_formation_position(20, 15);
        assert_eq!(enemy.x, 12);
        assert_eq!(enemy.y, 13);
    }

    #[test]
    fn test_enemy_damage_flash() {
        let mut enemy = Enemy::new_in_formation(10, 10, EnemyType::Basic, 0, (0, 0));
        assert!(!enemy.is_flashing());
        assert_eq!(enemy.damage_flash_frames, 0);

        // Take damage should trigger flash
        enemy.take_damage(5);
        assert!(enemy.is_flashing());
        assert_eq!(enemy.damage_flash_frames, 10);

        // Flash should decrease with updates
        enemy.update();
        assert_eq!(enemy.damage_flash_frames, 9);
        assert!(enemy.is_flashing());

        // Flash should eventually stop
        for _ in 0..9 {
            enemy.update();
        }
        assert_eq!(enemy.damage_flash_frames, 0);
        assert!(!enemy.is_flashing());
    }

    #[test]
    fn test_enemy_movement_state_initialization() {
        let enemy = Enemy::new_in_formation(10, 10, EnemyType::Fast, 0, (0, 0));
        assert_eq!(enemy.movement_state, MovementState::Descending);
        assert_eq!(enemy.movement_timer, 0);
        assert_eq!(enemy.target_x, None);
    }

    #[test]
    fn test_fast_enemy_zigzag_movement() {
        let mut enemy = Enemy::new_in_formation(40, 10, EnemyType::Fast, 0, (0, 0));
        enemy.formation_id = None; // Remove from formation to test individual movement
        
        let initial_x = enemy.x;
        
        // Update enough times to see zigzag pattern
        for _ in 0..25 {
            enemy.update();
        }
        
        // Enemy should have moved horizontally from zigzag pattern
        assert_ne!(enemy.x, initial_x);
        // Should be in either zigzag state (direction may have changed)
        assert!(matches!(enemy.movement_state, MovementState::ZigzagLeft | MovementState::ZigzagRight));
    }

    #[test]
    fn test_tank_enemy_pause_behavior() {
        let mut enemy = Enemy::new_in_formation(40, 10, EnemyType::Tank, 0, (0, 0));
        enemy.formation_id = None; // Remove from formation to test individual movement
        
        let initial_y = enemy.y;
        
        // Update to trigger pause state
        for _ in 0..40 {
            enemy.update();
        }
        
        // Should have paused at some point
        assert_eq!(enemy.movement_state, MovementState::Paused);
        assert!(enemy.y > initial_y); // Should have moved down some
    }

    // Property-based tests
    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_enemy_health_never_negative(
                enemy_type in prop::sample::select(vec![EnemyType::Basic, EnemyType::Fast, EnemyType::Tank]),
                damage_amounts in prop::collection::vec(0u8..30, 0..10)
            ) {
                let mut enemy = Enemy::new_in_formation(10, 10, enemy_type, 0, (0, 0));
                let initial_health = enemy.health;
                for damage in damage_amounts {
                    enemy.take_damage(damage);
                }
                prop_assert!(enemy.health <= initial_health);
            }
        }
    }
}
