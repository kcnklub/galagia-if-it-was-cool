/// Game entities for the space battle game

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeaponType {
    BasicGun,
    Sword,
    Bug,
}

impl WeaponType {
    pub fn get_name(&self) -> &'static str {
        match self {
            WeaponType::BasicGun => "Basic Gun",
            WeaponType::Sword => "Sword",
            WeaponType::Bug => "Bug",
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
}

impl Player {
    pub fn new(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            health: 100,
            fire_cooldown: 0,
            current_weapon: WeaponType::BasicGun,
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
        self.fire_cooldown = 5; // 5 frames between shots
    }

    pub fn update_cooldown(&mut self) {
        if self.fire_cooldown > 0 {
            self.fire_cooldown -= 1;
        }
    }

    pub fn take_damage(&mut self, damage: u8) {
        self.health = self.health.saturating_sub(damage);
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
        }
    }

    pub fn change_weapon(&mut self, weapon_type: WeaponType) {
        self.current_weapon = weapon_type;
    }
}

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
}

impl Enemy {
    pub fn new(x: u16, y: u16, enemy_type: EnemyType) -> Self {
        let health = match enemy_type {
            EnemyType::Basic => 10,
            EnemyType::Fast => 5,
            EnemyType::Tank => 20,
        };

        Self {
            x,
            y,
            health,
            enemy_type,
            fire_cooldown: 0,
        }
    }

    pub fn update(&mut self) {
        // Move down based on type
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

        if self.fire_cooldown % move_interval == 0 {
            self.y += speed;
        }

        self.fire_cooldown = self.fire_cooldown.wrapping_add(1);
    }

    pub fn can_fire(&self) -> bool {
        self.fire_cooldown % 30 == 0 // Fire every 30 frames
    }

    pub fn take_damage(&mut self, damage: u8) {
        self.health = self.health.saturating_sub(damage);
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
        7
    }

    pub fn get_height(&self) -> u16 {
        3
    }

    pub fn get_points(&self) -> u32 {
        match self.enemy_type {
            EnemyType::Basic => 10,
            EnemyType::Fast => 20,
            EnemyType::Tank => 30,
        }
    }
}

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

    pub fn update(&mut self) {
        // Update lifetime
        if let Some(ref mut lifetime) = self.lifetime {
            if *lifetime > 0 {
                *lifetime -= 1;
            }
        }

        // Update vertical position
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

        // Update horizontal position based on velocity
        if self.velocity_x != 0 {
            let new_x = self.x as i16 + self.velocity_x;
            if new_x >= 0 {
                self.x = new_x as u16;
            }
        }
    }

    pub fn is_out_of_bounds(&self, max_x: u16, max_y: u16) -> bool {
        // Check if lifetime expired
        if let Some(lifetime) = self.lifetime {
            if lifetime == 0 {
                return true;
            }
        }

        // Check bounds
        self.y == 0 || self.y >= max_y || self.x >= max_x
    }
}

#[derive(Debug, Clone)]
pub struct Pickup {
    pub x: u16,
    pub y: u16,
    pub weapon_type: WeaponType,
    pub frame_counter: u8,
}

impl Pickup {
    pub fn new(x: u16, y: u16, weapon_type: WeaponType) -> Self {
        Self {
            x,
            y,
            weapon_type,
            frame_counter: 0,
        }
    }

    pub fn update(&mut self) {
        // Pickups fall down very slowly (every 15 frames)
        self.frame_counter = self.frame_counter.wrapping_add(1);
        if self.frame_counter % 15 == 0 {
            self.y += 1;
        }
    }

    pub fn is_out_of_bounds(&self, max_y: u16) -> bool {
        self.y >= max_y
    }

    pub fn get_width(&self) -> u16 {
        1
    }

    pub fn get_height(&self) -> u16 {
        1
    }

    pub fn get_char(&self) -> char {
        match self.weapon_type {
            WeaponType::BasicGun => 'G',
            WeaponType::Sword => 'S',
            WeaponType::Bug => 'B',
        }
    }
}
