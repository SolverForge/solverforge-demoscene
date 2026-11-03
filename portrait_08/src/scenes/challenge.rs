// CHALLENGE: how a hard problem gets beaten. A small knight against a
// giant in a ruin: the club comes down on every accent of the riff. The
// first attempts end in a red flash and a higher attempt number; by the
// last one the knight reads every swing, rolls clear, and the giant goes
// down. Then a quiet hall with a typewriter: save, and on to the next door.
// Original imagery, not footage of any game.
use super::typed;
use crate::gfx::{ease_out, hash, mix, Frame, H, W};
use crate::timeline::{smooth, Now, BEAT};

const HALL_FROM: f32 = 16.0;
/// Failed attempts: (time the knight is caught, attempt number after it).
const DEATHS: [(f32, u32); 2] = [(5.0, 2), (10.0, 17)];
/// When the giant falls.
const FALL_AT: f32 = 14.0;

pub fn draw(f: &mut Frame, n: Now, t: f32) -> f32 {
    if n.local < HALL_FROM {
        let fade = smooth(n.local / 0.6) * smooth((HALL_FROM - n.local) / 0.35);
        ruin(f, n.local, t, fade);
    } else {
        let fade = smooth((n.local - HALL_FROM) / 0.6) * smooth((n.len - n.local) / 0.8);
        hall(f, n.local - HALL_FROM, t, fade);
    }
    1.0
}

/// One swing a beat-pair: the club lands on the riff's accents (every
/// second of the section).
const SWING: f32 = 2.0 * BEAT;
const GROUND: f32 = 1420.0;

fn impact_x(k: i32) -> f32 {
    if k % 2 == 0 {
        420.0
    } else {
        680.0
    }
}

fn ruin(f: &mut Frame, local: f32, t: f32, a: f32) {
    // The fight stops when the giant falls: hold the last swing.
    let fall = smooth((local - FALL_AT) / 1.4);
    let real = local;
    let local = local.min(FALL_AT);
    let k = (local / SWING).floor() as i32;
    let u = (local / SWING).fract();
    // Screen shake right after each impact.
    let since = u * SWING;
    let shake = if k >= 1 {
        (-since * 14.0).exp() * 12.0
    } else {
        0.0
    };
    let (ox, oy) = ((since * 90.0).sin() * shake, (since * 70.0).cos() * shake);

    f.gradient(&[0x0a0605, 0x1a0e09, 0x2a140b, 0x120806]);
    f.glow(540.0 + ox, 1100.0 + oy, 700.0, 0x7a2e10, 0.3 * a);
    // Broken arches behind.
    for i in 0..5 {
        let x = 60.0 + i as f32 * 240.0 + ox * 0.3;
        f.rect(x as i32, 520, x as i32 + 46, GROUND as i32, 0x120a08, a);
        f.ring(x + 120.0, 520.0, 120.0, 30.0, 0x120a08, a);
    }
    // Floor.
    for y in GROUND as i32..H as i32 {
        let d = (y as f32 - GROUND) / (H as f32 - GROUND);
        f.rect(0, y, W as i32, y + 1, mix(0x24140c, 0x0b0605, d), a);
    }

    // The giant sinks and fades as it falls.
    giant(f, local, ox, oy + 260.0 * fall * fall, a * (1.0 - fall));
    if fall > 0.0 {
        for i in 0..90 {
            let ph = ((real - FALL_AT) * 0.7 + hash(i, 9) * 0.3).min(1.0);
            let x = 300.0 + hash(i, 10) * 480.0;
            let y = 1300.0 - ph * (300.0 + 700.0 * hash(i, 11));
            f.disc(x, y, 3.0, 0xffb070, (1.0 - ph) * a);
        }
    }

    // The knight: rolls to the other side a quarter second before the hit.
    let from = impact_x(k);
    let to = impact_x(k + 1);
    let roll = ((u * SWING - (SWING - 0.45)) / 0.35).clamp(0.0, 1.0);
    let kx = from + (to - from) * ease_out(roll);
    knight(f, kx + ox, GROUND + oy, roll, a);

    // Dust where the club just landed: where the knight stood before he
    // rolled clear.
    if k >= 1 {
        let at = impact_x(k - 1);
        for i in 0..50 {
            let ang = -std::f32::consts::PI * hash(i, 1);
            let sp = 120.0 + 380.0 * hash(i, 2);
            let r = sp * since;
            let fall = 400.0 * since * since;
            let (x, y) = (
                at + ang.cos() * r + ox,
                GROUND + ang.sin() * r * 0.6 + fall + oy,
            );
            f.disc(
                x,
                y,
                3.0 + 4.0 * hash(i, 3),
                0x8a6a4a,
                (1.0 - since / 0.8).max(0.0) * a,
            );
        }
    }
    // Embers rising.
    for i in 0..120 {
        let ph = (t * (0.08 + 0.1 * hash(i, 4)) + hash(i, 5)).fract();
        let x = hash(i, 6) * W as f32 + (t * 1.5 + i as f32).sin() * 30.0;
        let y = H as f32 - ph * 1500.0;
        f.disc(x, y, 2.0, 0xff8a3a, (1.0 - ph) * 0.8 * a);
    }
    // Each failed attempt: a red flash to black, then back in.
    for &(at, _) in &DEATHS {
        let d = real - at;
        if (0.0..0.9).contains(&d) {
            let k = if d < 0.3 {
                d / 0.3
            } else {
                1.0 - (d - 0.3) / 0.6
            };
            f.rect(
                0,
                0,
                W as i32,
                H as i32,
                mix(0x5a0606, 0x000000, d / 0.9),
                k.clamp(0.0, 1.0) * a,
            );
        }
    }
    let attempt = DEATHS
        .iter()
        .rev()
        .find(|(at, _)| real >= *at + 0.3)
        .map(|d| d.1)
        .unwrap_or(1);
    let label = if real >= FALL_AT + 0.6 {
        format!("ATTEMPT {attempt}: DOWN")
    } else {
        format!("ATTEMPT {attempt}")
    };
    f.text_centered(W as i32 / 2, 1560, &label, 3, 0xffb070, 0.85 * a);
}

fn giant(f: &mut Frame, local: f32, ox: f32, oy: f32, a: f32) {
    let c = 0x050303;
    let rim = 0x5a2410;
    let (cx, top) = (540.0 + ox, 520.0 + oy);
    // Legs, planted wide.
    for side in [-1.0f32, 1.0] {
        let hip = (cx + side * 90.0, top + 640.0);
        let foot = (cx + side * 190.0, GROUND + oy);
        f.line(hip.0, hip.1, foot.0, foot.1, 110.0, c, a, 1.0);
    }
    // Torso: broad shoulders down to the waist, a rim of firelight.
    for dy in 0..560 {
        let k = dy as f32 / 560.0;
        let half = 250.0 - 130.0 * k.powf(0.8);
        let y = top + 120.0 + dy as f32;
        f.rect(
            (cx - half) as i32,
            y as i32,
            (cx + half) as i32,
            y as i32 + 1,
            c,
            a,
        );
        f.rect(
            (cx - half) as i32,
            y as i32,
            (cx - half) as i32 + 4,
            y as i32 + 1,
            rim,
            a,
        );
    }
    f.disc(cx - 230.0, top + 150.0, 80.0, c, a);
    f.disc(cx + 230.0, top + 150.0, 80.0, c, a);
    // Head and horns.
    f.disc(cx, top + 70.0, 80.0, c, a);
    for side in [-1.0f32, 1.0] {
        for i in 0..30 {
            let u = i as f32 / 29.0;
            let x = cx + side * (60.0 + u * 110.0);
            let y = top + 30.0 - u * 120.0 + u * u * 60.0;
            f.disc(x, y, 22.0 * (1.0 - u) + 3.0, c, a);
        }
    }
    for ex in [-26.0, 26.0] {
        f.disc(cx + ex, top + 70.0, 7.0, 0xff5a1a, a);
        f.glow(cx + ex, top + 70.0, 30.0, 0xff5a1a, 0.6 * a);
    }
    // The free arm hangs on the left; the right arm swings the club.
    let sh = (cx - 250.0, top + 170.0);
    f.line(sh.0, sh.1, sh.0 - 60.0, sh.1 + 420.0, 70.0, c, a, 1.0);
    // From resting where it last landed, up overhead, down on the spot the
    // knight stands on now.
    let k = (local / SWING).floor() as i32;
    let u = (local / SWING).fract();
    let shoulder = (cx + 250.0, top + 170.0);
    let aim = |x: f32| {
        let d = (x + ox - shoulder.0, GROUND - 20.0 + oy - shoulder.1);
        (d.1.atan2(d.0), (d.0 * d.0 + d.1 * d.1).sqrt())
    };
    let (rest_ang, rest_len) = aim(impact_x(k - 1));
    let (slam_ang, slam_len) = aim(impact_x(k));
    let raised_ang = -std::f32::consts::FRAC_PI_2 - 0.4;
    let raise = smooth(u / 0.6);
    let slam = smooth((u - 0.8) / 0.2);
    let up = rest_ang + (raised_ang - rest_ang) * raise;
    let ang = up + (slam_ang - up) * slam;
    let reach = rest_len + (slam_len - rest_len) * smooth(u);
    let hand = (
        shoulder.0 + ang.cos() * reach * 0.55,
        shoulder.1 + ang.sin() * reach * 0.55,
    );
    let tip = (
        shoulder.0 + ang.cos() * reach,
        shoulder.1 + ang.sin() * reach,
    );
    f.line(shoulder.0, shoulder.1, hand.0, hand.1, 80.0, c, a, 1.0);
    // The club thickens toward its head.
    for s in 0..12 {
        let k = s as f32 / 11.0;
        let (x, y) = (hand.0 + (tip.0 - hand.0) * k, hand.1 + (tip.1 - hand.1) * k);
        f.disc(x, y, 30.0 + 40.0 * k, c, a);
    }
    f.disc(tip.0, tip.1, 72.0, rim, 0.25 * a);
}

/// The knight: helm, shield, sword; tucks into a ball while rolling.
fn knight(f: &mut Frame, x: f32, ground: f32, roll: f32, a: f32) {
    // A shade lighter than the giant, with a bright firelit rim.
    let c = 0x2a1a12;
    let rim = 0xffb070;
    let s = 1.7;
    let tuck = (std::f32::consts::PI * roll).sin();
    let r = |x0: f32, y0: f32, x1: f32, y1: f32, f: &mut Frame, col: u32, al: f32| {
        f.rect(
            (x + x0 * s) as i32,
            (ground - y1 * s) as i32,
            (x + x1 * s) as i32,
            (ground - y0 * s) as i32,
            col,
            al,
        );
    };
    if tuck > 0.3 {
        f.disc(x, ground - 36.0 * s, 36.0 * s, c, a);
        f.ring(x, ground - 36.0 * s, 36.0 * s, 2.0, rim, 0.6 * a);
        return;
    }
    r(-18.0, 40.0, 18.0, 110.0, f, c, a);
    r(-16.0, 0.0, -4.0, 40.0, f, c, a);
    r(4.0, 0.0, 16.0, 40.0, f, c, a);
    f.disc(x, ground - 130.0 * s, 20.0 * s, c, a);
    r(-10.0, 128.0, 14.0, 134.0, f, rim, 0.7 * a);
    f.disc(x - 30.0 * s, ground - 80.0 * s, 26.0 * s, c, a);
    f.ring(x - 30.0 * s, ground - 80.0 * s, 26.0 * s, 2.0, rim, 0.6 * a);
    f.line(
        x + 20.0 * s,
        ground - 80.0 * s,
        x + 50.0 * s,
        ground - 190.0 * s,
        5.0,
        0xc8c0b8,
        a,
        1.0,
    );
    f.line(
        x - 18.0 * s,
        ground - 110.0 * s,
        x - 18.0 * s,
        ground - 40.0 * s,
        2.0,
        rim,
        0.5 * a,
        1.0,
    );
}

/// The hall, one-point perspective toward a door at the far end.
fn hall(f: &mut Frame, local: f32, t: f32, a: f32) {
    let vp = (540.0, 820.0);
    // Lamp flicker.
    let flick = 0.8 + 0.2 * ((t * 23.0).sin() * (t * 7.0).sin()).abs()
        - if hash((t * 8.0) as i32, 3) > 0.93 {
            0.4
        } else {
            0.0
        };
    let sting =
        ((local - 7.5) / 0.15).clamp(0.0, 1.0) * (1.0 - ((local - 7.65) / 0.4).clamp(0.0, 1.0));
    let light = flick * a;
    f.clear(0x050605);
    // Walls, floor, ceiling as quads around the far rectangle.
    let (fx0, fy0, fx1, fy1) = (430.0, 700.0, 650.0, 960.0);
    f.par_rows(0, H, |y, row| {
        for (x, p) in row.iter_mut().enumerate() {
            let (xf, yf) = (x as f32, y as f32);
            let inside = xf > fx0 && xf < fx1 && yf > fy0 && yf < fy1;
            if inside {
                continue;
            }
            // Distance: how far toward the vanishing point this pixel is.
            let dx = (xf - vp.0) / (if xf < vp.0 { vp.0 - fx0 } else { fx1 - vp.0 });
            let dy = (yf - vp.1) / (if yf < vp.1 { vp.1 - fy0 } else { fy1 - vp.1 });
            let m = dx.abs().max(dy.abs());
            let depth = 1.0 / m;
            let c = if dy.abs() > dx.abs() && yf > vp.1 {
                // Floor: checker in depth and across.
                let u = (xf - vp.0) / (yf - vp.1) * 3.0;
                let v = 2400.0 / (yf - vp.1);
                let chk = ((u.floor() + v.floor()) as i32).rem_euclid(2);
                if chk == 0 {
                    0x2c2a22
                } else {
                    0x16150f
                }
            } else if dy.abs() > dx.abs() {
                0x121410
            } else {
                // Walls: wainscot below, paper above, doors.
                let h = (yf - vp.1) / (xf - vp.0).abs() * 220.0;
                let along = 160.0 / (xf - vp.0).abs() * 60.0;
                let door = (along * 0.6).fract() < 0.22 && h > -110.0 && h < 120.0;
                if door {
                    0x2a1a10
                } else if h > 40.0 {
                    0x2a2416
                } else {
                    0x1d2219
                }
            };
            let lamp = (1.0 / (1.0 + (depth - 1.6).powi(2) * 0.8)) * light;
            *p = mix(0x000000, c, (0.25 + 0.75 * lamp).min(1.0));
        }
    });
    // The far door, dark, and something in front of it, closer each beat.
    // A dim light through the far doorway, so what stands in it shows.
    f.rect(
        fx0 as i32,
        fy0 as i32,
        fx1 as i32,
        fy1 as i32,
        0x3a3f30,
        a * (0.6 + 0.4 * flick),
    );
    f.glow(vp.0, fy1 - 60.0, 260.0, 0x5a6048, 0.25 * a);
    let near = smooth(local / 8.0) * 0.6 + sting * 0.4;
    let (zx, zy) = (vp.0, fy1 + near * 380.0);
    let s = 0.5 + near * 2.2;
    let sway = (t * 1.6).sin() * 8.0 * s;
    f.disc(zx + sway, zy - 210.0 * s, 22.0 * s, 0x0b0c09, a);
    f.rect(
        (zx - 30.0 * s + sway) as i32,
        (zy - 190.0 * s) as i32,
        (zx + 30.0 * s + sway) as i32,
        (zy - 60.0 * s) as i32,
        0x0b0c09,
        a,
    );
    f.line(
        zx - 30.0 * s + sway,
        zy - 170.0 * s,
        zx - 60.0 * s + sway,
        zy - 100.0 * s,
        10.0 * s,
        0x0b0c09,
        a,
        1.0,
    );
    f.line(
        zx + 30.0 * s + sway,
        zy - 170.0 * s,
        zx + 66.0 * s + sway,
        zy - 120.0 * s,
        10.0 * s,
        0x0b0c09,
        a,
        1.0,
    );
    f.rect(
        (zx - 22.0 * s) as i32,
        (zy - 60.0 * s) as i32,
        (zx - 6.0 * s) as i32,
        zy as i32,
        0x0b0c09,
        a,
    );
    f.rect(
        (zx + 6.0 * s) as i32,
        (zy - 60.0 * s) as i32,
        (zx + 22.0 * s) as i32,
        zy as i32,
        0x0b0c09,
        a,
    );
    // The ceiling lamp.
    f.line(540.0, 380.0, 540.0, 470.0, 2.0, 0x302a20, a, 1.0);
    f.disc(540.0, 480.0, 16.0, 0xffe8b0, light);
    f.glow(540.0, 480.0, 220.0, 0xffcf80, 0.3 * light);

    typewriter(f, local, a);
    if sting > 0.0 {
        f.rect(0, 0, W as i32, H as i32, 0xd8d0c0, 0.25 * sting);
    }
}

fn typewriter(f: &mut Frame, local: f32, a: f32) {
    let (x0, y0) = (190, 1260);
    // Paper.
    f.rect(x0 + 160, y0 - 230, x0 + 540, y0 + 20, 0xd8d0b8, a);
    let line1 = typed("SAVE GAME?", 0.6, 9.0, local);
    let line2 = typed("> YES", 2.6, 6.0, local);
    f.text(x0 + 200, y0 - 190, &line1, 3, 0x2a2420, a);
    f.text(x0 + 200, y0 - 140, &line2, 3, 0x6a1a14, a);
    // Body.
    f.rect(x0, y0, x0 + 700, y0 + 300, 0x15130f, a);
    f.rect(x0 - 20, y0 - 20, x0 + 720, y0 + 20, 0x221e18, a);
    f.frame_rect(x0, y0, x0 + 700, y0 + 300, 0x3a3428, a);
    // Keys, three rows, a key pressed with each typed letter.
    let typing = ((local - 0.6) * 9.0) as i32;
    for row in 0..3 {
        for col in 0..10 - row {
            let kx = x0 + 90 + row * 30 + col * 56;
            let ky = y0 + 90 + row * 64;
            let pressed = (0..10).contains(&typing) && (typing * 7 + 3) % 27 == row * 10 + col;
            let dy = if pressed { 4 } else { 0 };
            f.disc(kx as f32, (ky + dy) as f32, 20.0, 0x0a0908, a);
            f.ring(kx as f32, (ky + dy) as f32, 20.0, 2.5, 0x8a8070, a);
        }
    }
}
