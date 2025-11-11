use super::projectile::{Projectile, ProjectileOwner, ProjectileType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeaponType {
    BasicGun,
    Sword,
    Bug,
    Bomber,
}

impl WeaponType {
    pub fn get_name(&self) -> &'static str {
        match self {
            WeaponType::BasicGun => "Basic Gun",
            WeaponType::Sword => "Sword",
            WeaponType::Bug => "Bug",
            WeaponType::Bomber => "The Bomber",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub x: u16,
    pub y: u16,
    pub health: u8,
    pub fire_cooldown: u8,
    pub current_weapon: WeaponType,
    pub damage_flash_frames: u8,
}

impl Player {
    pub fn new(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            health: 100,
            fire_cooldown: 0,
            current_weapon: WeaponType::BasicGun,
            damage_flash_frames: 0,
        }
    }

    pub fn move_left(&mut self, min_x: u16) {
        if self.x > min_x {
            self.x -= 1;
        }
    }

    pub fn move_right(&mut self, max_x: u16) {
        if self.x < max_x {
            self.x += 1;
        }
    }

    pub fn move_up(&mut self, min_y: u16) {
        if self.y > min_y {
            self.y -= 1;
        }
    }

    pub fn move_down(&mut self, max_y: u16) {
        if self.y < max_y {
            self.y += 1;
        }
    }

    pub fn can_fire(&self) -> bool {
        self.fire_cooldown == 0
    }

    pub fn reset_cooldown(&mut self) {
        // Different weapons have different fire rates
        self.fire_cooldown = match self.current_weapon {
            WeaponType::BasicGun => 10,
            WeaponType::Sword => 8,
            WeaponType::Bug => 10,
            WeaponType::Bomber => 30, // Much slower fire rate for bomber (0.5 seconds)
        };
    }

    pub fn update_cooldown(&mut self) {
        if self.fire_cooldown > 0 {
            self.fire_cooldown -= 1;
        }
        if self.damage_flash_frames > 0 {
            self.damage_flash_frames -= 1;
        }
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
        vec![" /^\\ ", "<|||>", " ||| "]
    }

    pub fn get_width(&self) -> u16 {
        5
    }

    pub fn get_height(&self) -> u16 {
        3
    }

    /// Attempts to fire projectile(s) if cooldown allows
    /// Returns Vec of projectiles if fire was successful, empty vec otherwise
    pub fn try_fire(&mut self) -> Vec<Projectile> {
        if !self.can_fire() {
            return vec![];
        }

        self.reset_cooldown();
        let center_x = self.x + self.get_width() / 2;
        let fire_y = self.y;

        match self.current_weapon {
            WeaponType::BasicGun => {
                // Single straight shot
                vec![Projectile::new_with_type(
                    center_x,
                    fire_y,
                    ProjectileOwner::Player,
                    ProjectileType::Bullet,
                    0,
                    None,
                )]
            }
            WeaponType::Sword => {
                // Arc slash in front of ship with limited lifetime
                vec![Projectile::new_with_type(
                    center_x,
                    fire_y.saturating_sub(1),
                    ProjectileOwner::Player,
                    ProjectileType::Slash,
                    0,
                    Some(10), // Slash lasts 10 frames
                )]
            }
            WeaponType::Bug => {
                // Dual angled shots in V-pattern
                vec![
                    // Left diagonal shot
                    Projectile::new_with_type(
                        center_x,
                        fire_y,
                        ProjectileOwner::Player,
                        ProjectileType::BugShot,
                        -1, // Move left
                        None,
                    ),
                    // Right diagonal shot
                    Projectile::new_with_type(
                        center_x,
                        fire_y,
                        ProjectileOwner::Player,
                        ProjectileType::BugShot,
                        1, // Move right
                        None,
                    ),
                ]
            }
            WeaponType::Bomber => {
                // Slow-moving bomb that explodes after a short time
                vec![Projectile::new_with_damage(
                    center_x,
                    fire_y,
                    ProjectileOwner::Player,
                    ProjectileType::BomberProjectile,
                    0,
                    Some(90), // Bomb lasts 90 frames (~1.5 seconds) before exploding
                    5,        // Direct hit does only 5 damage, explosion does AoE damage
                )]
            }
        }
    }

    pub fn change_weapon(&mut self, weapon_type: WeaponType) {
        self.current_weapon = weapon_type;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_new() {
        let player = Player::new(40, 20);
        assert_eq!(player.x, 40);
        assert_eq!(player.y, 20);
        assert_eq!(player.health, 100);
        assert_eq!(player.fire_cooldown, 0);
        assert_eq!(player.current_weapon, WeaponType::BasicGun);
    }

    #[test]
    fn test_player_movement_left() {
        let mut player = Player::new(10, 10);
        player.move_left(0);
        assert_eq!(player.x, 9);

        // Test boundary
        player.x = 0;
        player.move_left(0);
        assert_eq!(player.x, 0);
    }

    #[test]
    fn test_player_movement_right() {
        let mut player = Player::new(10, 10);
        player.move_right(79);
        assert_eq!(player.x, 11);

        // Test boundary
        player.x = 79;
        player.move_right(79);
        assert_eq!(player.x, 79);
    }

    #[test]
    fn test_player_movement_up() {
        let mut player = Player::new(10, 10);
        player.move_up(0);
        assert_eq!(player.y, 9);

        // Test boundary
        player.y = 0;
        player.move_up(0);
        assert_eq!(player.y, 0);
    }

    #[test]
    fn test_player_movement_down() {
        let mut player = Player::new(10, 10);
        player.move_down(23);
        assert_eq!(player.y, 11);

        // Test boundary
        player.y = 23;
        player.move_down(23);
        assert_eq!(player.y, 23);
    }

    #[test]
    fn test_player_fire_cooldown() {
        let mut player = Player::new(10, 10);
        assert!(player.can_fire());

        player.reset_cooldown();
        assert_eq!(player.fire_cooldown, 10);
        assert!(!player.can_fire());

        // Test cooldown update
        for _ in 0..10 {
            player.update_cooldown();
        }
        assert!(player.can_fire());
    }

    #[test]
    fn test_player_take_damage() {
        let mut player = Player::new(10, 10);
        player.take_damage(30);
        assert_eq!(player.health, 70);
        assert!(player.is_alive());

        player.take_damage(80);
        assert_eq!(player.health, 0);
        assert!(!player.is_alive());
    }

    #[test]
    fn test_player_try_fire_basic_gun() {
        let mut player = Player::new(10, 10);
        let projectiles = player.try_fire();
        assert_eq!(projectiles.len(), 1);
        assert_eq!(projectiles[0].owner, ProjectileOwner::Player);
        assert_eq!(projectiles[0].projectile_type, ProjectileType::Bullet);
    }

    #[test]
    fn test_player_try_fire_sword() {
        let mut player = Player::new(10, 10);
        player.change_weapon(WeaponType::Sword);
        let projectiles = player.try_fire();
        assert_eq!(projectiles.len(), 1);
        assert_eq!(projectiles[0].projectile_type, ProjectileType::Slash);
        assert_eq!(projectiles[0].lifetime, Some(10));
    }

    #[test]
    fn test_player_try_fire_bug() {
        let mut player = Player::new(10, 10);
        player.change_weapon(WeaponType::Bug);
        let projectiles = player.try_fire();
        assert_eq!(projectiles.len(), 2);
        assert_eq!(projectiles[0].velocity_x, -1);
        assert_eq!(projectiles[1].velocity_x, 1);
    }

    #[test]
    fn test_player_cooldown_prevents_firing() {
        let mut player = Player::new(10, 10);
        player.try_fire();
        let projectiles = player.try_fire();
        assert_eq!(projectiles.len(), 0);
    }

    #[test]
    fn test_player_damage_flash() {
        let mut player = Player::new(10, 10);
        assert!(!player.is_flashing());
        assert_eq!(player.damage_flash_frames, 0);

        // Take damage should trigger flash
        player.take_damage(10);
        assert!(player.is_flashing());
        assert_eq!(player.damage_flash_frames, 10);

        // Flash should decrease with updates
        player.update_cooldown();
        assert_eq!(player.damage_flash_frames, 9);
        assert!(player.is_flashing());

        // Flash should eventually stop
        for _ in 0..9 {
            player.update_cooldown();
        }
        assert_eq!(player.damage_flash_frames, 0);
        assert!(!player.is_flashing());
    }

    // Property-based tests
    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_player_stays_in_bounds_x(
                initial_x in 0u16..80,
                moves in prop::collection::vec(prop::bool::ANY, 0..100)
            ) {
                let mut player = Player::new(initial_x, 10);
                for move_right in moves {
                    if move_right {
                        player.move_right(79);
                    } else {
                        player.move_left(0);
                    }
                }
                prop_assert!(player.x <= 79);
            }

            #[test]
            fn test_player_stays_in_bounds_y(
                initial_y in 0u16..24,
                moves in prop::collection::vec(prop::bool::ANY, 0..100)
            ) {
                let mut player = Player::new(10, initial_y);
                for move_down in moves {
                    if move_down {
                        player.move_down(23);
                    } else {
                        player.move_up(0);
                    }
                }
                prop_assert!(player.y <= 23);
            }

            #[test]
            fn test_player_health_never_negative(
                initial_health in 0u8..100,
                damage_amounts in prop::collection::vec(0u8..50, 0..10)
            ) {
                let mut player = Player::new(10, 10);
                player.health = initial_health;
                for damage in damage_amounts {
                    player.take_damage(damage);
                }
                // Health should be 0 or positive (saturating_sub ensures this)
                prop_assert!(player.health <= initial_health);
            }
        }
    }
}
