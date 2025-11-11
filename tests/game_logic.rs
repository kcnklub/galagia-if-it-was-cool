/// Integration tests for game logic
///
/// These tests verify interactions between different game entities
/// and core gameplay mechanics like collision detection and scoring.
use simple::{Enemy, EnemyType, Player, Projectile, ProjectileOwner, WeaponType};

/// Helper function to check if two rectangles collide (AABB collision detection)
#[allow(clippy::too_many_arguments)]
fn check_collision(x1: u16, y1: u16, w1: u16, h1: u16, x2: u16, y2: u16, w2: u16, h2: u16) -> bool {
    x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2
}

#[test]
fn test_player_projectile_hits_enemy() {
    let enemy = Enemy::new_in_formation(20, 10, EnemyType::Basic, 0, (0, 0));
    let projectile = Projectile::new(22, 12, ProjectileOwner::Player);

    // Check collision
    let collision = check_collision(
        projectile.x,
        projectile.y,
        1,
        1,
        enemy.x,
        enemy.y,
        enemy.get_width(),
        enemy.get_height(),
    );

    assert!(collision);
}

#[test]
fn test_enemy_projectile_hits_player() {
    let player = Player::new(40, 20);
    let projectile = Projectile::new(42, 21, ProjectileOwner::Enemy);

    // Check collision
    let collision = check_collision(
        projectile.x,
        projectile.y,
        1,
        1,
        player.x,
        player.y,
        player.get_width(),
        player.get_height(),
    );

    assert!(collision);
}

#[test]
fn test_no_collision_when_far_apart() {
    let enemy = Enemy::new_in_formation(20, 10, EnemyType::Basic, 0, (0, 0));
    let projectile = Projectile::new(50, 12, ProjectileOwner::Player);

    // Check collision
    let collision = check_collision(
        projectile.x,
        projectile.y,
        1,
        1,
        enemy.x,
        enemy.y,
        enemy.get_width(),
        enemy.get_height(),
    );

    assert!(!collision);
}

#[test]
fn test_enemy_destroyed_gives_correct_points() {
    let enemy_types = vec![
        (EnemyType::Basic, 10),
        (EnemyType::Fast, 20),
        (EnemyType::Tank, 30),
    ];

    for (enemy_type, expected_points) in enemy_types {
        let enemy = Enemy::new_in_formation(20, 10, enemy_type, 0, (0, 0));
        assert_eq!(enemy.get_points(), expected_points);
    }
}

#[test]
fn test_enemy_takes_damage_and_dies() {
    let mut enemy = Enemy::new_in_formation(20, 10, EnemyType::Basic, 0, (0, 0));
    let projectile = Projectile::new(22, 12, ProjectileOwner::Player);

    // Simulate hit - Basic enemy has 15 health, projectile does 10 damage
    assert!(enemy.is_alive());
    enemy.take_damage(projectile.damage);

    // With 15 health and 10 damage, health becomes 5
    assert_eq!(enemy.health, 5);
    assert!(enemy.is_alive());

    // Take another hit to kill
    enemy.take_damage(projectile.damage);
    assert_eq!(enemy.health, 0);
    assert!(!enemy.is_alive());
}

#[test]
fn test_player_takes_damage_and_dies() {
    let mut player = Player::new(40, 20);
    let projectile = Projectile::new(42, 21, ProjectileOwner::Enemy);

    // Simulate multiple hits
    assert!(player.is_alive());
    for _ in 0..10 {
        player.take_damage(projectile.damage);
    }
    assert_eq!(player.health, 0);
    assert!(!player.is_alive());
}

#[test]
fn test_player_weapon_switch_changes_projectile_count() {
    let mut player = Player::new(40, 20);

    // Basic gun fires 1 projectile
    player.change_weapon(WeaponType::BasicGun);
    let projectiles = player.try_fire();
    assert_eq!(projectiles.len(), 1);

    // Reset cooldown for next test
    player.fire_cooldown = 0;

    // Sword fires 1 projectile (slash)
    player.change_weapon(WeaponType::Sword);
    let projectiles = player.try_fire();
    assert_eq!(projectiles.len(), 1);

    player.fire_cooldown = 0;

    // Bug weapon fires 2 projectiles
    player.change_weapon(WeaponType::Bug);
    let projectiles = player.try_fire();
    assert_eq!(projectiles.len(), 2);
}

#[test]
fn test_multiple_projectiles_move_independently() {
    let mut player_proj = Projectile::new(10, 10, ProjectileOwner::Player);
    let mut enemy_proj = Projectile::new(20, 10, ProjectileOwner::Enemy);

    player_proj.update();
    enemy_proj.update();

    // Player projectile moves up
    assert_eq!(player_proj.y, 9);
    // Enemy projectile moves down
    assert_eq!(enemy_proj.y, 11);
}

#[test]
fn test_enemy_survives_partial_damage() {
    let mut enemy = Enemy::new_in_formation(20, 10, EnemyType::Tank, 0, (0, 0));
    assert_eq!(enemy.health, 30);

    enemy.take_damage(5);
    assert!(enemy.is_alive());
    assert_eq!(enemy.health, 25);

    enemy.take_damage(5);
    assert!(enemy.is_alive());
    assert_eq!(enemy.health, 20);
}

#[test]
fn test_formation_enemy_follows_position() {
    let mut enemy = Enemy::new_in_formation(10, 10, EnemyType::Basic, 0, (8, 4));

    // Update formation position
    enemy.update_formation_position(20, 15);

    // Enemy should be at center + offset
    assert_eq!(enemy.x, 28); // 20 + 8
    assert_eq!(enemy.y, 19); // 15 + 4
}

#[test]
fn test_player_cooldown_limits_fire_rate() {
    let mut player = Player::new(40, 20);

    // First shot should work
    let projectiles = player.try_fire();
    assert_eq!(projectiles.len(), 1);

    // Immediate second shot should be blocked
    let projectiles = player.try_fire();
    assert_eq!(projectiles.len(), 0);

    // After cooldown expires, should be able to fire again
    for _ in 0..10 {
        player.update_cooldown();
    }
    let projectiles = player.try_fire();
    assert_eq!(projectiles.len(), 1);
}
