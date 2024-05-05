turbo::cfg! {r#"
    name = "Darth Vader's Chip Battle"
    version = "0.0.1"
    author = "@khubert"
    description = "Survive waves of tortilla chips and limes using a lightsaber and force lightning! Darth Vader helmet by @XFrequencyX on DeviantArt."
    [settings]
    resolution = [320, 240]
"#}

turbo::init! {
    struct GameState {
        player_x: f32,
        player_y: f32,
        target_angle: i32,
        current_angle: i32,
        calorie_count: u32,
        force_lightning_cooldown: u32,
        game_over: bool,
        game_tick: usize,
        enemies: Vec<struct Enemy {
            x: f32,
            y: f32,
            radius: f32,
            enemy_type: enum EnemyType {
                Chip,
                Lime,
            },
        }>,
    } = {
        Self {
            player_x: 160.0,
            player_y: 120.0,
            target_angle: 0,
            current_angle: 0,
            calorie_count: 0,
            force_lightning_cooldown: 0,
            game_over: false,
            game_tick: 0,
            enemies: vec![],
        }
    }
}

fn rect_collision(rect_left: i32, rect_top: i32, rect_width: i32, rect_height: i32, rect_rotation: i32, circle_x: f32, circle_y: f32, circle_radius: f32) -> bool {
    let rect_center_x = rect_left + rect_width / 2;
    let rect_center_y = rect_top + rect_height / 2;
    let relative_circle_x = circle_x - rect_center_x as f32;
    let relative_circle_y = circle_y - rect_center_y as f32;
    let rotated_circle_x = relative_circle_x * (rect_rotation as f32).to_radians().cos() - relative_circle_y * (rect_rotation as f32).to_radians().sin();
    let rotated_circle_y = relative_circle_x * (rect_rotation as f32).to_radians().sin() + relative_circle_y * (rect_rotation as f32).to_radians().cos();
    let closest_x = rotated_circle_x.max(-rect_width as f32 / 2.0).min(rect_width as f32 / 2.0);
    let closest_y = rotated_circle_y.max(-rect_height as f32 / 2.0).min(rect_height as f32 / 2.0);
    let distance_x = rotated_circle_x - closest_x;
    let distance_y = rotated_circle_y - closest_y;
    let distance_squared = distance_x.powi(2) + distance_y.powi(2);
    distance_squared <= circle_radius.powi(2)
}

turbo::go! {
    let mut state = GameState::load();

    if state.calorie_count >= 2000 {
        state.game_over = true;
    }

    if state.game_over {
        text!("Game Over!", x = 130, y = 120, font = Font::L, color = 0xff0000ff);

        if gamepad(0).a.just_pressed() {
            state.game_tick = tick();
            state.calorie_count = 0;
            state.target_angle = 0;
            state.current_angle = 0;
            state.force_lightning_cooldown = 0;
            state.game_over = false;
            state.player_x = 160.0;
            state.player_y = 120.0;
        }

        state.save();
        return;
    }

    if gamepad(0).left.pressed() {
        state.player_x -= 3.;
    }
    if gamepad(0).right.pressed() {
        state.player_x += 3.;
    }
    if gamepad(0).up.pressed() {
        state.player_y -= 3.;
    }
    if gamepad(0).down.pressed() {
        state.player_y += 3.;
    }

    if gamepad(0).a.just_pressed() {
        state.target_angle += 180;
    }

    if state.current_angle != state.target_angle {
        let diff = state.target_angle - state.current_angle;
        let step = diff / 5;
        state.current_angle += step;
        if (state.current_angle - state.target_angle).abs() < 1 {
            state.current_angle = state.target_angle;
        }
    }

    if gamepad(0).b.just_pressed() && state.force_lightning_cooldown == 0 {
        state.enemies.retain(|enemy| {
            let distance = ((state.player_x - enemy.x).powi(2) + (state.player_y - enemy.y).powi(2)).sqrt();

            // Draw force lightning
            path!(
                start = (state.player_x as i32, state.player_y as i32),
                end = (enemy.x as i32, enemy.y as i32),
                color = 0x4444ffff
            );

            distance > 100.0
        });
        state.force_lightning_cooldown = 300; // Cooldown for 5 seconds
    }

    if state.force_lightning_cooldown > 0 {
        state.force_lightning_cooldown -= 1;
    }

    if rand() % 30 == 0 {
        let enemy_type = if rand() % 3 > 0 { EnemyType::Chip } else { EnemyType::Lime };
        let side = rand() % 4;
        let (x, y) = match side {
            0 => ((rand() % 320) as f32, 0.0), // Top side
            1 => ((rand() % 320) as f32, 240.0), // Bottom side
            2 => (0.0, (rand() % 240) as f32), // Left side
            3 => (320.0, (rand() % 240) as f32), // Right side
            _ => unreachable!(),
        };
        let radius = if enemy_type == EnemyType::Chip {
            16.0
        } else {
            8.0
        };
        let enemy = Enemy {
            x: x,
            y: y,
            radius: radius,
            enemy_type: enemy_type,
        };
        state.enemies.push(enemy);
    }

    let ls_rect_x = (state.player_x - 12.5 + (state.current_angle as f32).to_radians().cos() * 25.0) as i32;
    let ls_rect_y = (state.player_y + (state.current_angle as f32).to_radians().sin() * 25.0) as i32;

    state.enemies.retain_mut(|enemy| {
        let angle = (state.player_y - enemy.y).atan2(state.player_x - enemy.x);
        let speed = 1.0;
        enemy.x += speed * angle.cos();
        enemy.y += speed * angle.sin();

        if (state.player_x - enemy.x).abs() < 10.0 && (state.player_y - enemy.y).abs() < 10.0 {
            match enemy.enemy_type {
                EnemyType::Chip => state.calorie_count += 25,
                EnemyType::Lime => state.calorie_count += 50,
            }
            return false;
        }

        return !rect_collision(ls_rect_x, ls_rect_y, 30, 5, state.current_angle, enemy.x, enemy.y, enemy.radius);
    });

    sprite!("darth", x = state.player_x as i32 - 16, y = state.player_y as i32 - 16, w = 32, h = 32);
    rect!(x = ls_rect_x, y = ls_rect_y, w = 30, h = 5, rotate = state.current_angle, color = 0xff0000ff);

    for enemy in &state.enemies {
        let color = match enemy.enemy_type {
            EnemyType::Chip => 0xc2b280ff,
            EnemyType::Lime => 0x00ff00ff,
        };
        circ!(x = enemy.x as i32, y = enemy.y as i32, d = enemy.radius as u32, color = color);
    }

    text!(&format!("Calories: {}", state.calorie_count), x = 10, y = 10, font = Font::M, color = 0xffffffff);

    let time = ((tick() - state.game_tick) / 60) as u32;
    if time > 99 {
        text!(&format!("Time: {}s", time), x = 264, y = 10, font = Font::M, color = 0xffffffff);
    } else {
        text!(&format!("Time: {}s", time), x = 270, y = 10, font = Font::M, color = 0xffffffff);
    }

    if state.force_lightning_cooldown > 0 {
        let cooldown_seconds = state.force_lightning_cooldown / 60;
        text!(&format!("Force Lightning Cooldown: {}s", cooldown_seconds), x = 100, y = 10, font = Font::M, color = 0xffffffff);
    }

    state.save();
}
