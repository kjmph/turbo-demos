
turbo::cfg! {r#"
    name = "Star Wars Blaster Challenge"
    version = "0.0.1"
    author = "@khubert"
    description = "Defend the galaxy! X-wing by @PACrankyDM and Tie-fighter by @TheRealMessyRoomGuy1 at DeviantArt."
    [settings]
    resolution = [320, 240]
"#}

turbo::init! {
    struct GameState {
        frame: u32,
        player_x: f32,
        player_y: f32,
        lives: i32,
        shots: Vec<struct Shot {
            x: f32,
            y: f32,
            dy: f32,
            hit: bool,
        }>,
        enemies: Vec<struct Enemy {
            x: f32,
            y: f32,
            dy: f32,
        }>,
        score: u32,
        last_enemy_spawn: u32,
    } = {
        Self {
            frame: 0,
            player_x: 160.0,
            player_y: 200.0,
            lives: 3,
            shots: vec![],
            enemies: vec![],
            score: 0,
            last_enemy_spawn: 0,
        }
    }
}

turbo::go! {
    let mut state = GameState::load();
    clear(0x111144ff);

    if gamepad(0).left.pressed() {
        state.player_x -= 3.;
    }

    if gamepad(0).right.pressed() {
        state.player_x += 3.;
    }

    if gamepad(0).a.just_pressed() {
        if state.lives <= 0 {
            state.lives = 3;
            state.player_x = 160.0;
            state.player_y = 200.0;
            state.score = 0;
        }

        state.shots.push(Shot {
            x: state.player_x + 24.0,
            y: state.player_y - 10.0,
            dy: -5.0,
            hit: false,
        });
        state.shots.push(Shot {
            x: state.player_x - 24.0,
            y: state.player_y - 10.0,
            dy: -5.0,
            hit: false,
        });
    }

    state.shots.retain_mut(|shot| {
        shot.y += shot.dy;
        if shot.y > 0.0 && !shot.hit {
            circ!(x = shot.x as i32, y = shot.y as i32, d = 2, color = 0xffff00ff);
            true
        } else {
            false
        }
    });

    if state.frame - state.last_enemy_spawn > (61 - state.score.min(60)) {
        state.enemies.push(Enemy {
            x: (rand() % 320) as f32,
            y: -10.0,
            dy: (rand() % 3 + 1) as f32,
        });
        state.last_enemy_spawn = state.frame;
    }

    state.enemies.retain_mut(|enemy| {
        enemy.y += enemy.dy;
        if enemy.y < 240.0 {
            sprite!("tie-fighter", x = enemy.x as i32 - 16, y = enemy.y as i32 - 16, w = 32, h = 32);
            true
        } else {
            false
        }
    });

    for shot in &mut state.shots {
        state.enemies.retain_mut(|enemy| {
            let dx = shot.x - enemy.x;
            let dy = shot.y - enemy.y;
            let miss = (dx * dx + dy * dy).sqrt() > 16.0;
            if !miss {
                state.score += 1;
                shot.hit = true;
            }
            miss
        });
    }

    state.enemies.retain_mut(|enemy| {
        let dx = state.player_x - enemy.x;
        let dy = state.player_y - enemy.y;
        let miss = (dx * dx + dy * dy).sqrt() > 24.0;
        if !miss {
            state.player_x = 160.0;
            state.player_y = 200.0;
            state.lives -= 1;
        }
        miss
    });

    if state.lives <= 0 {
        text!("Game Over", x = 125, y = 100, font = Font::L, color = 0xff0000ff);
    } else {
        if state.lives > 0 {
            for i in 0..state.lives {
                text!("o", x = 300 - (i as i32) * 20, y = 10, font = Font::L, color = 0xff0000ff)
            }
        }

        sprite!("x-wing", x = state.player_x as i32 - 24, y = state.player_y as i32 - 14, w = 48, h = 28);
    }

    text!(&format!("Score: {}", state.score), x = 10, y = 10, font = Font::L, color = 0xffffffff);

    state.frame += 1;
    state.save();
}
