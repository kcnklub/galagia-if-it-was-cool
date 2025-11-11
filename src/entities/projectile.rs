#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectileOwner {
    Player,
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectileType {
    Bullet,
    Slash,
    BugShot,
    BomberProjectile,
}

#[derive(Debug, Clone)]
pub struct Projectile {
    pub x: u16,
    pub y: u16,
    pub owner: ProjectileOwner,
    pub damage: u8,
    pub projectile_type: ProjectileType,
    pub velocity_x: i16,
    pub lifetime: Option<u8>,
}

impl Projectile {
    pub fn new(x: u16, y: u16, owner: ProjectileOwner) -> Self {
        let damage = match owner {
            ProjectileOwner::Player => 10,
            ProjectileOwner::Enemy => 10,
        };

        Self {
            x,
            y,
            owner,
            damage,
            projectile_type: ProjectileType::Bullet,
            velocity_x: 0,
            lifetime: None,
        }
    }

    pub fn new_with_type(
        x: u16,
        y: u16,
        owner: ProjectileOwner,
        projectile_type: ProjectileType,
        velocity_x: i16,
        lifetime: Option<u8>,
    ) -> Self {
        let damage = match owner {
            ProjectileOwner::Player => 10,
            ProjectileOwner::Enemy => 10,
        };

        Self {
            x,
            y,
            owner,
            damage,
            projectile_type,
            velocity_x,
            lifetime,
        }
    }

    pub fn new_with_damage(
        x: u16,
        y: u16,
        owner: ProjectileOwner,
        projectile_type: ProjectileType,
        velocity_x: i16,
        lifetime: Option<u8>,
        damage: u8,
    ) -> Self {
        Self {
            x,
            y,
            owner,
            damage,
            projectile_type,
            velocity_x,
            lifetime,
        }
    }

    pub fn update(&mut self) {
        // Update lifetime
        if let Some(ref mut lifetime) = self.lifetime
            && *lifetime > 0 {
                *lifetime -= 1;
            }

        // Update vertical position
        // Bomber projectiles move slower (every 3rd frame)
        let should_move = if self.projectile_type == ProjectileType::BomberProjectile {
            // Use lifetime to determine movement (move on frames where lifetime % 3 == 0)
            self.lifetime.is_none_or(|l| l % 3 == 0)
        } else {
            true
        };

        if should_move {
            match self.owner {
                ProjectileOwner::Player => {
                    if self.y > 0 {
                        self.y -= 1;
                    }
                }
                ProjectileOwner::Enemy => {
                    self.y += 1;
                }
            }
        }

        // Update horizontal position based on velocity
        if self.velocity_x != 0 {
            let new_x = self.x as i16 + self.velocity_x;
            if new_x >= 0 {
                self.x = new_x as u16;
            }
        }
    }

    pub fn is_out_of_bounds(&self, min_x: u16, max_x: u16, max_y: u16) -> bool {
        // Check if lifetime expired
        if let Some(lifetime) = self.lifetime
            && lifetime == 0 {
                return true;
            }

        // Check bounds
        self.y == 0 || self.y >= max_y || self.x < min_x || self.x >= max_x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projectile_new() {
        let projectile = Projectile::new(10, 10, ProjectileOwner::Player);
        assert_eq!(projectile.x, 10);
        assert_eq!(projectile.y, 10);
        assert_eq!(projectile.owner, ProjectileOwner::Player);
        assert_eq!(projectile.damage, 10);
    }

    #[test]
    fn test_player_projectile_moves_up() {
        let mut projectile = Projectile::new(10, 10, ProjectileOwner::Player);
        projectile.update();
        assert_eq!(projectile.y, 9);
    }

    #[test]
    fn test_enemy_projectile_moves_down() {
        let mut projectile = Projectile::new(10, 10, ProjectileOwner::Enemy);
        projectile.update();
        assert_eq!(projectile.y, 11);
    }

    #[test]
    fn test_projectile_horizontal_velocity() {
        let mut projectile = Projectile::new_with_type(
            10,
            10,
            ProjectileOwner::Player,
            ProjectileType::BugShot,
            2,
            None,
        );
        projectile.update();
        assert_eq!(projectile.x, 12);
        assert_eq!(projectile.y, 9);
    }

    #[test]
    fn test_projectile_out_of_bounds() {
        let projectile = Projectile::new(0, 0, ProjectileOwner::Player);
        assert!(projectile.is_out_of_bounds(0, 80, 24));

        let projectile = Projectile::new(10, 24, ProjectileOwner::Enemy);
        assert!(projectile.is_out_of_bounds(0, 80, 24));
    }

    #[test]
    fn test_projectile_lifetime() {
        let mut projectile = Projectile::new_with_type(
            10,
            10,
            ProjectileOwner::Player,
            ProjectileType::Slash,
            0,
            Some(3),
        );

        assert!(!projectile.is_out_of_bounds(0, 80, 24));
        projectile.update();
        assert_eq!(projectile.lifetime, Some(2));
        projectile.update();
        assert_eq!(projectile.lifetime, Some(1));
        projectile.update();
        assert_eq!(projectile.lifetime, Some(0));
        assert!(projectile.is_out_of_bounds(0, 80, 24));
    }

    // Property-based tests
    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_projectile_moves_in_correct_direction(
                initial_x in 5u16..75,
                initial_y in 5u16..19,
                owner in prop::sample::select(vec![ProjectileOwner::Player, ProjectileOwner::Enemy])
            ) {
                let mut projectile = Projectile::new(initial_x, initial_y, owner);
                projectile.update();

                match owner {
                    ProjectileOwner::Player => {
                        // Player projectiles move up (y decreases)
                        prop_assert!(projectile.y < initial_y || initial_y == 0);
                    }
                    ProjectileOwner::Enemy => {
                        // Enemy projectiles move down (y increases)
                        prop_assert!(projectile.y > initial_y);
                    }
                }
            }
        }
    }
}
