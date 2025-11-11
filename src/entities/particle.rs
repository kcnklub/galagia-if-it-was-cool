#[derive(Debug, Clone)]
pub struct Particle {
    pub x: u16,
    pub y: u16,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub lifetime: u8,
    pub char: char,
}

impl Particle {
    pub fn new(x: u16, y: u16, velocity_x: i16, velocity_y: i16, lifetime: u8, char: char) -> Self {
        Self {
            x,
            y,
            velocity_x,
            velocity_y,
            lifetime,
            char,
        }
    }

    pub fn update(&mut self) {
        // Decrease lifetime
        if self.lifetime > 0 {
            self.lifetime -= 1;
        }

        // Update position based on velocity
        if self.velocity_x != 0 {
            let new_x = self.x as i16 + self.velocity_x;
            if new_x >= 0 {
                self.x = new_x as u16;
            }
        }

        if self.velocity_y != 0 {
            let new_y = self.y as i16 + self.velocity_y;
            if new_y >= 0 {
                self.y = new_y as u16;
            }
        }
    }

    pub fn is_dead(&self) -> bool {
        self.lifetime == 0
    }

    pub fn is_out_of_bounds(&self, min_x: u16, max_x: u16, max_y: u16) -> bool {
        self.y >= max_y || self.x < min_x || self.x >= max_x
    }
}

/// Creates an explosion particle effect at the given position
pub fn create_explosion_particles(center_x: u16, center_y: u16) -> Vec<Particle> {
    let mut particles = Vec::new();

    // Create particles in 8 directions (cardinal + diagonal)
    let directions = [
        (0, -1),   // Up
        (1, -1),   // Up-Right
        (1, 0),    // Right
        (1, 1),    // Down-Right
        (0, 1),    // Down
        (-1, 1),   // Down-Left
        (-1, 0),   // Left
        (-1, -1),  // Up-Left
    ];

    for (dx, dy) in directions.iter() {
        particles.push(Particle::new(
            center_x,
            center_y,
            dx * 1,
            dy * 1,
            6, // Particles last 6 frames (~0.1 seconds)
            '*',
        ));
    }

    // Add one central particle
    particles.push(Particle::new(
        center_x,
        center_y,
        0,
        0,
        4, // Brief flash
        'o',
    ));

    particles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_new() {
        let particle = Particle::new(10, 10, 1, -1, 10, '*');
        assert_eq!(particle.x, 10);
        assert_eq!(particle.y, 10);
        assert_eq!(particle.velocity_x, 1);
        assert_eq!(particle.velocity_y, -1);
        assert_eq!(particle.lifetime, 10);
        assert_eq!(particle.char, '*');
    }

    #[test]
    fn test_particle_update_position() {
        let mut particle = Particle::new(10, 10, 2, -1, 10, '*');
        particle.update();
        assert_eq!(particle.x, 12);
        assert_eq!(particle.y, 9);
        assert_eq!(particle.lifetime, 9);
    }

    #[test]
    fn test_particle_lifetime_expires() {
        let mut particle = Particle::new(10, 10, 0, 0, 2, '*');
        assert!(!particle.is_dead());
        particle.update();
        assert!(!particle.is_dead());
        particle.update();
        assert!(particle.is_dead());
    }

    #[test]
    fn test_particle_out_of_bounds() {
        let particle = Particle::new(100, 50, 0, 0, 10, '*');
        assert!(particle.is_out_of_bounds(0, 80, 24));

        let particle = Particle::new(10, 10, 0, 0, 10, '*');
        assert!(!particle.is_out_of_bounds(0, 80, 24));
    }

    #[test]
    fn test_create_explosion_particles() {
        let particles = create_explosion_particles(10, 10);
        // 8 directions (cardinal + diagonal) + 1 central particle = 9 particles
        assert_eq!(particles.len(), 9);

        // All particles should start at the same position
        for particle in particles.iter() {
            assert_eq!(particle.x, 10);
            assert_eq!(particle.y, 10);
        }
    }
}
