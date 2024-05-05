use std::cmp;

turbo::cfg! {r#"
    name = "Catch Me If You Can"
    version = "0.0.1"
    author = "@khubert"
    description = "A simple chase & evade game"
"#}

turbo::init! {
    struct GameState {
        p0_hits: i32,
        p1_hits: i32,
        p0_cps: i32,
        p1_cps: i32,
    } = {
        Self {
            p0_hits: 0,
            p1_hits: 0,
            p0_cps: 0,
            p1_cps: 0,
        }
    }
}

struct Rectangle {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

fn rect_collision(rect_a: &Rectangle, rect_b: &Rectangle) -> bool {
    let horizontal_overlap = (rect_b.x - rect_a.x - rect_a.w) * (rect_a.x - rect_b.x - rect_b.w) >= 0;
    let vertical_overlap = (rect_b.y - rect_a.y - rect_a.h) * (rect_a.y - rect_b.y - rect_b.h) >= 0;

    horizontal_overlap && vertical_overlap
}

fn interpolate_color(a: i32, b: i32) -> u32 {
    let min_diff = -8;
    let max_diff = 8;
    let magenta = (0xFF, 0x00, 0xFF);
    let cyan = (0x00, 0xFF, 0xFF);

    let diff = (a - b).clamp(min_diff, max_diff);

    let normalized_diff = (diff - min_diff) as f32 / (max_diff - min_diff) as f32;

    let r = ((1.0 - normalized_diff) * cyan.0 as f32 + normalized_diff * magenta.0 as f32) as u8;
    let g = ((1.0 - normalized_diff) * cyan.1 as f32 + normalized_diff * magenta.1 as f32) as u8;
    let b = ((1.0 - normalized_diff) * cyan.2 as f32 + normalized_diff * magenta.2 as f32) as u8;

    ((r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | 0xff) as u32
}

turbo::go! {
    let mut state = GameState::load();
    let m0 = mouse(0);
    let m1 = mouse(1);

    // Magic values (6,7) are related to Font::L size
    let rect_m0 = Rectangle {
        x: m0.position[0] - 6,
        y: m0.position[1] - 7,
        w: 6,
        h: 7,
    };

    let rect_m1 = Rectangle {
        x: m1.position[0] - 6,
        y: m1.position[1] - 7,
        w: 6,
        h: 7,
    };

    let collided = rect_collision(&rect_m0, &rect_m1);

    if m0.left.just_pressed() && collided {
        state.p1_hits += 1;
    }

    if m1.left.just_pressed() && collided {
        state.p0_hits += 1;
    }

    // Restart game with a mini CPS game..
    if state.p0_hits >= 10 || state.p1_hits >= 10 {
        if m0.left.just_pressed() {
            state.p0_cps += 1;
        }

        if m1.left.just_pressed() {
            state.p1_cps += 1;
        }

        clear(interpolate_color(state.p0_cps, state.p1_cps));
    }

    // Paint the current leader on top
    if state.p0_hits > state.p1_hits {
        text!(
            &cmp::min(state.p0_hits, 9).to_string(),
            x = rect_m0.x,
            y = rect_m0.y,
            color = 0xff00ffff,
            font = Font::L
        );

        text!(
            &cmp::min(state.p1_hits, 9).to_string(),
            x = rect_m1.x,
            y = rect_m1.y,
            color = 0xffffffff,
            font = Font::L
        );
    } else {
        text!(
            &cmp::min(state.p1_hits, 9).to_string(),
            x = rect_m1.x,
            y = rect_m1.y,
            color = 0xffffffff,
            font = Font::L
        );

        text!(
            &cmp::min(state.p0_hits, 9).to_string(),
            x = rect_m0.x,
            y = rect_m0.y,
            color = 0xff00ffff,
            font = Font::L
        );
    }

    if state.p0_hits >= 10 {
        text!(
            "Player 2 wins!",
            x = 80,
            y = 128,
            color = 0xffffffff,
            font = Font::L
        );
    }

    if state.p1_hits >= 10 {
        text!(
            "Player 1 wins!",
            x = 80,
            y = 128,
            color = 0xffffffff,
            font = Font::L
        );
    }

    if (state.p0_cps - state.p1_cps).abs() > 8 {
        state.p0_hits = 0;
        state.p1_hits = 0;
        state.p0_cps = 0;
        state.p0_cps = 0;
        clear(0x000000ff);
    }

    state.save();
}
