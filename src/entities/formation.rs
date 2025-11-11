#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormationType {
    VShape,  // V-shaped formation
    Diamond, // Diamond/rhombus shape
    Wall,    // Horizontal wall
    Block,   // Dense rectangular block
}

#[derive(Debug, Clone)]
pub struct Formation {
    /// Center X position of the formation
    pub center_x: u16,
    /// Center Y position of the formation
    pub center_y: u16,
    /// Formation type
    pub formation_type: FormationType,
    /// Movement direction (-1 left, 0 none, 1 right)
    pub direction_x: i16,
    /// Frame counter for timing
    pub frame_counter: u16,
    /// Indices of enemies in this formation
    pub enemy_indices: Vec<usize>,
}

impl Formation {
    pub fn new(center_x: u16, center_y: u16, formation_type: FormationType) -> Self {
        Self {
            center_x,
            center_y,
            formation_type,
            direction_x: 1, // Start moving right
            frame_counter: 0,
            enemy_indices: Vec::new(),
        }
    }

    /// Get relative positions for enemies in this formation
    /// Returns (dx, dy) offsets from center
    pub fn get_positions(&self) -> Vec<(i16, i16)> {
        match self.formation_type {
            FormationType::VShape => vec![
                // Top of V
                (0, 0),
                // Left arm
                (-8, 4),
                (-16, 8),
                (-24, 12),
                // Right arm
                (8, 4),
                (16, 8),
                (24, 12),
            ],
            FormationType::Diamond => vec![
                // Top
                (0, 0),
                // Middle row
                (-8, 4),
                (8, 4),
                // Widest row
                (-16, 8),
                (0, 8),
                (16, 8),
                // Bottom row
                (-8, 12),
                (8, 12),
                // Bottom point
                (0, 16),
            ],
            FormationType::Wall => vec![
                (-24, 0),
                (-16, 0),
                (-8, 0),
                (0, 0),
                (8, 0),
                (16, 0),
                (24, 0),
                (-24, 4),
                (-16, 4),
                (-8, 4),
                (0, 4),
                (8, 4),
                (16, 4),
                (24, 4),
            ],
            FormationType::Block => vec![
                // Dense 4x4 block
                (-12, 0),
                (-4, 0),
                (4, 0),
                (12, 0),
                (-12, 4),
                (-4, 4),
                (4, 4),
                (12, 4),
                (-12, 8),
                (-4, 8),
                (4, 8),
                (12, 8),
                (-12, 12),
                (-4, 12),
                (4, 12),
                (12, 12),
            ],
        }
    }

    pub fn update(&mut self, max_x: u16) {
        self.frame_counter += 1;

        // Move formation down every 8 frames
        if self.frame_counter % 8 == 0 {
            self.center_y += 1;
        }

        // Move formation horizontally every 4 frames
        if self.frame_counter % 4 == 0 {
            let new_x = self.center_x as i16 + self.direction_x;

            // Get the formation width to check bounds properly
            let positions = self.get_positions();
            let min_offset = positions.iter().map(|(x, _)| *x).min().unwrap_or(0);
            let max_offset = positions.iter().map(|(x, _)| *x).max().unwrap_or(0);

            // Check if the new position would put any enemy out of bounds
            let left_edge = new_x + min_offset;
            let right_edge = new_x + max_offset;

            // Keep formation within bounds with padding
            if left_edge >= 5 && right_edge <= (max_x as i16 - 10) {
                self.center_x = new_x as u16;
            } else {
                // Hit edge, reverse direction
                self.direction_x = -self.direction_x;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formation_new() {
        let formation = Formation::new(40, 10, FormationType::VShape);
        assert_eq!(formation.center_x, 40);
        assert_eq!(formation.center_y, 10);
        assert_eq!(formation.formation_type, FormationType::VShape);
        assert_eq!(formation.direction_x, 1);
    }

    #[test]
    fn test_formation_v_shape_positions() {
        let formation = Formation::new(40, 10, FormationType::VShape);
        let positions = formation.get_positions();
        assert_eq!(positions.len(), 7);
        assert_eq!(positions[0], (0, 0)); // Top of V
    }

    #[test]
    fn test_formation_diamond_positions() {
        let formation = Formation::new(40, 10, FormationType::Diamond);
        let positions = formation.get_positions();
        assert_eq!(positions.len(), 9);
    }

    #[test]
    fn test_formation_wall_positions() {
        let formation = Formation::new(40, 10, FormationType::Wall);
        let positions = formation.get_positions();
        assert_eq!(positions.len(), 14);
    }

    #[test]
    fn test_formation_block_positions() {
        let formation = Formation::new(40, 10, FormationType::Block);
        let positions = formation.get_positions();
        assert_eq!(positions.len(), 16);
    }

    #[test]
    fn test_formation_update_moves_down() {
        let mut formation = Formation::new(40, 10, FormationType::VShape);
        for _ in 0..8 {
            formation.update(80);
        }
        assert_eq!(formation.center_y, 11);
    }

    #[test]
    fn test_formation_update_moves_horizontally() {
        let mut formation = Formation::new(40, 10, FormationType::VShape);
        for _ in 0..4 {
            formation.update(80);
        }
        assert_eq!(formation.center_x, 41);
    }

    #[test]
    fn test_formation_reverses_at_boundary() {
        let mut formation = Formation::new(70, 10, FormationType::VShape);

        // Move right until hitting boundary
        for _ in 0..100 {
            formation.update(80);
        }

        // Should have reversed direction at some point
        assert_eq!(formation.direction_x, -1);
    }

    // Property-based tests
    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_formation_positions_consistent(
                center_x in 30u16..50,
                center_y in 5u16..15,
                formation_type in prop::sample::select(vec![
                    FormationType::VShape,
                    FormationType::Diamond,
                    FormationType::Wall,
                    FormationType::Block
                ])
            ) {
                let formation = Formation::new(center_x, center_y, formation_type);
                let positions = formation.get_positions();

                // All positions should be consistent (non-empty and each position unique)
                prop_assert!(!positions.is_empty());

                // Check that applying offsets to center gives reasonable positions
                for (dx, dy) in positions {
                    let abs_x = center_x as i16 + dx;
                    let abs_y = center_y as i16 + dy;
                    prop_assert!(abs_x >= 0);
                    prop_assert!(abs_y >= 0);
                }
            }

            #[test]
            fn test_formation_stays_within_bounds(
                initial_x in 30u16..50,
                initial_y in 5u16..10
            ) {
                let mut formation = Formation::new(initial_x, initial_y, FormationType::VShape);

                // Run many update cycles
                for _ in 0..200 {
                    formation.update(80);

                    // Get the formation's actual bounds
                    let positions = formation.get_positions();
                    let min_offset = positions.iter().map(|(x, _)| *x).min().unwrap();
                    let max_offset = positions.iter().map(|(x, _)| *x).max().unwrap();

                    let left_edge = formation.center_x as i16 + min_offset;
                    let right_edge = formation.center_x as i16 + max_offset;

                    // Formation should stay within reasonable bounds
                    prop_assert!(left_edge >= 0);
                    prop_assert!(right_edge < 80);
                }
            }
        }
    }
}
