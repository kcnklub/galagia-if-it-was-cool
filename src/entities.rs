/// Game entities for the space battle game

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub x: u16,
    pub y: u16,
    pub health: u8,
    pub fire_cooldown: u8,
}

impl Player {
    pub fn new(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            health: 100,
            fire_cooldown: 0,
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

    pub fn get_sprite(&self) -> &'static str {
        " /^\\ \n<|||>\n ||| "
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

    pub fn get_sprite(&self) -> &'static str {
        match self.enemy_type {
            EnemyType::Basic => " \\|/ \n{===}\n /_\\ ",
            EnemyType::Fast => " <> \n<||>\n <> ",
            EnemyType::Tank => "[===]\n|###|\n[===]",
        }
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

#[derive(Debug, Clone)]
pub struct Projectile {
    pub x: u16,
    pub y: u16,
    pub owner: ProjectileOwner,
    pub damage: u8,
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
        }
    }

    pub fn update(&mut self) {
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

    pub fn is_out_of_bounds(&self, max_y: u16) -> bool {
        self.y == 0 || self.y >= max_y
    }

    pub fn get_sprite(&self) -> char {
        match self.owner {
            ProjectileOwner::Player => '^',
            ProjectileOwner::Enemy => 'v',
        }
    }
}
