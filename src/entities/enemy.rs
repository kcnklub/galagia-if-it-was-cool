#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyType {
    Basic,
    Fast,
    Tank,
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

        // Move down based on type (for non-formation enemies)
        let speed = match self.enemy_type {
            EnemyType::Basic => 1,
            EnemyType::Fast => 1,
            EnemyType::Tank => 1,
        };

        // Move down every few frames - slowed down significantly
        let move_interval = match self.enemy_type {
            EnemyType::Basic => 8, // Move every 8 frames
            EnemyType::Fast => 5,  // Move every 5 frames (still faster)
            EnemyType::Tank => 10, // Move every 10 frames (slowest)
        };

        if self.fire_cooldown.is_multiple_of(move_interval) {
            self.y += speed;
        }

        self.fire_cooldown = self.fire_cooldown.wrapping_add(1);
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
        }
    }

    pub fn get_width(&self) -> u16 {
        match self.enemy_type {
            EnemyType::Basic => 7,
            EnemyType::Fast => 8,  // Sprite size for dark-fighter
            EnemyType::Tank => 8,  // Sprite size for dark-tanker
        }
    }

    pub fn get_height(&self) -> u16 {
        match self.enemy_type {
            EnemyType::Basic => 3,
            EnemyType::Fast => 5,  // Sprite size for dark-fighter
            EnemyType::Tank => 5,  // Sprite size for dark-tanker
        }
    }

    pub fn get_points(&self) -> u32 {
        match self.enemy_type {
            EnemyType::Basic => 10,
            EnemyType::Fast => 20,
            EnemyType::Tank => 30,
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
