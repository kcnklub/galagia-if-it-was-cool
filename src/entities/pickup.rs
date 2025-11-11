use super::player::WeaponType;

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
            WeaponType::Bomber => 'X',
        }
    }
}
