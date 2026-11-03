// WORLDS: old online worlds. A hooded undead frost mage walks toward us
// down a night road, away from a ruined keep; the camera backs away with
// him, so the road and the trees recede. Herbs glow by the road and go
// into the bags; then the trade channel scrolls and the herbs are sold.
// Original imagery in the spirit of the genre, not footage of any game.
use super::typed;
use crate::gfx::{ease_out, hash, mix, Frame, H, W};
use crate::timeline::{smooth, Now};

const SKY: [u32; 4] = [0x070b1c, 0x0d1634, 0x1c1f45, 0x2a2140];
const INK: u32 = 0x05060c;
const FROST: u32 = 0x9ad7ff;
const GOLD: u32 = 0xf2c14e;

/// Herbs the mage gathers: (seconds into the section, kind, sell price in
/// copper). The coin counter is the sum of what has been sold.
const HERBS: [(f32, usize, u32); 6] = [
    (9.0, 0, 35),
    (10.4, 1, 60),
    (11.8, 0, 35),
    (13.2, 2, 90),
    (14.4, 1, 60),
    (15.4, 3, 140),
];
const HERB_NAMES: [&str; 4] = ["PEACEBLOOM", "SILVERLEAF", "MAGEROYAL", "BRIARTHORN"];
const POTIONS: [(f32, usize); 2] = [(12.6, 0), (15.8, 1)];
const SALES_FROM: f32 = 18.2;
const SALE_EVERY: f32 = 0.7;

const CHAT: [(f32, &str, u32); 6] = [
    (16.4, "[TRADE] WTB SILVERLEAF, 20", 0xf3d9a4),
    (17.2, "[TRADE] WTS MINOR MANA POTION", 0xf3d9a4),
    (18.0, "[GUILD] LFM CATACOMBS, NEED A HEALER", 0x7ee08a),
    (19.0, "[TRADE] PST FOR HERBS, FAIR PRICES", 0xf3d9a4),
    (20.2, "[GUILD] SAME TIME TOMORROW?", 0x7ee08a),
    (21.4, "[WHISPER] THANKS FOR THE POTIONS", 0xe7a0ff),
];

pub fn draw(f: &mut Frame, n: Now, t: f32) -> f32 {
    let fade = n.envelope(1.0, 0.8);
    f.gradient(&SKY);
    for i in 0..360 {
        let (x, y) = (hash(i, 1) * W as f32, 380.0 + hash(i, 2) * 700.0);
        let tw = 0.5 + 0.5 * (t * (1.0 + 2.0 * hash(i, 3)) + hash(i, 4) * 9.0).sin();
        f.blend(
            x as i32,
            y as i32,
            0xdfe8ff,
            (0.2 + 0.6 * hash(i, 5)) * tw * fade,
        );
    }
    // A large moon behind the keep.
    f.glow(700.0, 640.0, 330.0, 0x5d6aa8, 0.35 * fade);
    f.disc(700.0, 640.0, 120.0, 0xe9ecf6, 0.95 * fade);
    f.disc(668.0, 612.0, 20.0, 0xc9cfe2, 0.5 * fade);
    f.disc(740.0, 690.0, 14.0, 0xc9cfe2, 0.45 * fade);

    // The mage walks toward us down the road: the camera backs away, so
    // road, trees and hills recede up toward the horizon.
    let walk = -n.local * 9.0;
    ridge(f, 980.0, 160.0, 0.0011, 3, 0x141836, fade, walk * 0.15);
    keep(f, 760.0, 900.0, fade);
    ridge(f, 1130.0, 120.0, 0.0024, 7, 0x0e1128, fade, walk * 0.35);
    ridge(f, 1300.0, 90.0, 0.004, 11, 0x090b1c, fade, walk * 0.7);
    road(f, walk, fade);
    trees(f, walk, fade);

    // Herbs by the road: they glow, the mage reaches, they fly to the bags.
    let bag_in = smooth((n.local - 8.0) / 0.8);
    for (i, &(at, kind, _)) in HERBS.iter().enumerate() {
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let (hx, hy) = (540.0 + side * 170.0, 1300.0);
        let k = (n.local - at) / 0.8;
        if (-1.6..0.0).contains(&k) {
            let pulse = 0.6 + 0.4 * (t * 6.0).sin();
            herb(
                f,
                hx,
                hy + 64.0 - (k + 1.6) * 40.0,
                kind,
                fade * smooth(k + 1.6),
                pulse,
            );
        } else if (0.0..1.0).contains(&k) {
            let (sx, sy) = slot_xy(i);
            let e = ease_out(k);
            let (x, y) = (
                hx + (sx - hx) * e,
                hy + (sy - hy) * e - 120.0 * (std::f32::consts::PI * e).sin(),
            );
            herb(f, x, y, kind, fade, 1.0);
        }
    }

    mage(f, n.local, t, fade);

    // Bags and professions.
    if bag_in > 0.0 {
        bags(f, n.local, bag_in * fade);
    }
    // The trade channel.
    let chat_in = smooth((n.local - 16.0) / 0.6);
    if chat_in > 0.0 {
        chat(f, n.local, chat_in * fade);
    }
    1.0
}

/// A noisy ridge line, filled below; `drift` shifts it as the camera moves.
#[allow(clippy::too_many_arguments)]
fn ridge(f: &mut Frame, base: f32, amp: f32, freq: f32, seed: i32, c: u32, a: f32, drift: f32) {
    for x in 0..W as i32 {
        let xf = x as f32;
        let mut h = 0.0;
        let mut k = 1.0;
        for o in 0..4 {
            h += (xf * freq * k + seed as f32 * 1.7 + o as f32).sin() * amp / k;
            k *= 2.1;
        }
        let top = base + drift.clamp(-200.0, 200.0) - h.abs();
        f.rect(x, top as i32, x + 1, H as i32, c, a);
    }
}

/// A ruined keep with a broken tower and a few cold-lit windows.
fn keep(f: &mut Frame, cx: f32, base: f32, a: f32) {
    let c = 0x0b0d22;
    f.rect(
        cx as i32 - 90,
        base as i32 - 120,
        cx as i32 + 90,
        base as i32 + 120,
        c,
        a,
    );
    f.rect(
        cx as i32 - 30,
        base as i32 - 300,
        cx as i32 + 30,
        base as i32,
        c,
        a,
    );
    // Broken top: a jagged edge.
    for dx in -30..30 {
        let jag = (hash(dx / 6, 3) * 40.0) as i32;
        f.rect(
            cx as i32 + dx,
            base as i32 - 330 + jag,
            cx as i32 + dx + 1,
            base as i32 - 300,
            c,
            a,
        );
    }
    for i in 0..4 {
        let x = cx as i32 - 90 + i * 50;
        f.rect(x, base as i32 - 140, x + 24, base as i32 - 120, c, a);
    }
    for (x, y) in [(-6, -250), (-50, -60), (40, -40), (-6, -170)] {
        f.rect(
            cx as i32 + x,
            base as i32 + y,
            cx as i32 + x + 10,
            base as i32 + y + 18,
            0x7fc8ff,
            0.6 * a,
        );
        f.glow(
            cx + x as f32 + 5.0,
            base + y as f32 + 9.0,
            30.0,
            FROST,
            0.15 * a,
        );
    }
}

fn road(f: &mut Frame, walk: f32, a: f32) {
    for y in 1150..H as i32 {
        let depth = (y as f32 - 1150.0) / (H as f32 - 1150.0);
        let w = 40.0 + depth * 420.0;
        let bend = ((y as f32 + walk * 2.0) * 0.004).sin() * 60.0 * (1.0 - depth);
        let cx = 540.0 + bend;
        let stones = if (((y as f32 + walk * 6.0) * 0.12).floor() as i32).rem_euclid(2) == 0 {
            0.06
        } else {
            0.0
        };
        let c = mix(0x1d1c2a, 0x34324a, depth * 0.6 + stones);
        f.rect((cx - w) as i32, y, (cx + w) as i32, y + 1, c, a);
    }
}

fn trees(f: &mut Frame, walk: f32, a: f32) {
    for i in 0..8 {
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let y = (hash(i, 7) * 900.0 + walk * 6.0).rem_euclid(900.0) + 1150.0;
        let depth = (y - 1150.0) / 770.0;
        let x = 540.0 + side * (120.0 + depth * 470.0 + hash(i, 8) * 60.0);
        let h = 80.0 + depth * 340.0;
        let (bx, by) = (x, y);
        f.line(bx, by, bx, by - h, 3.0 + depth * 8.0, INK, a, 1.0);
        for b in 0..4 {
            let k = 0.35 + b as f32 * 0.15;
            let dir = if b % 2 == 0 { 1.0 } else { -1.0 };
            let (sx, sy) = (bx, by - h * k);
            f.line(
                sx,
                sy,
                sx + dir * h * 0.35,
                sy - h * 0.22,
                2.0 + depth * 4.0,
                INK,
                a,
                1.0,
            );
        }
    }
}

fn herb(f: &mut Frame, x: f32, y: f32, kind: usize, a: f32, glow: f32) {
    let c = [0xf7e7a1, 0xd9e6ff, 0xb98cff, 0xffb37a][kind];
    f.glow(x, y, 46.0, c, 0.25 * glow * a);
    for leaf in 0..5 {
        let ang = leaf as f32 * 1.25 + kind as f32;
        let (dx, dy) = (ang.cos() * 16.0, ang.sin() * 9.0 - 6.0);
        f.disc(x + dx, y + dy, 7.0, 0x3f8f4a, a);
    }
    f.disc(x, y - 10.0, 7.0, c, a);
}

/// The mage: hooded robe, thin undead arms, glowing eyes, a staff whose
/// orb throws frost when casting.
fn mage(f: &mut Frame, local: f32, t: f32, a: f32) {
    let (cx, base) = (540.0, 1330.0);
    let step = (local * 3.2).sin();
    let bob = step.abs() * 6.0;
    let y = base - bob;
    // Robe: a tall triangle with a hood.
    for dy in 0..300 {
        let k = dy as f32 / 300.0;
        let half = 30.0 + 70.0 * k + step * 6.0 * k;
        let yy = y - 300.0 + dy as f32;
        f.rect(
            (cx - half) as i32,
            yy as i32,
            (cx + half) as i32,
            yy as i32 + 1,
            0x111428,
            a,
        );
        // Moonlight catches the right edge of the robe.
        f.rect(
            (cx + half - 5.0) as i32,
            yy as i32,
            (cx + half) as i32,
            yy as i32 + 1,
            0x5d6aa8,
            0.7 * a,
        );
    }
    f.disc(cx + 3.0, y - 322.0, 46.0, 0x5d6aa8, 0.7 * a);
    f.disc(cx, y - 320.0, 46.0, 0x111428, a);
    f.disc(cx, y - 314.0, 32.0, 0x020205, a);
    for ex in [-12.0, 12.0] {
        f.disc(cx + ex, y - 318.0, 4.0, FROST, a);
        f.glow(cx + ex, y - 318.0, 14.0, FROST, 0.6 * a);
    }
    // Trim along the hem.
    f.rect(
        (cx - 100.0) as i32,
        (y - 4.0) as i32,
        (cx + 100.0) as i32,
        y as i32 + 2,
        0x3a5b8c,
        a,
    );
    // Staff and orb, raised while casting.
    let casting = ((local - 8.0) / 8.0).clamp(0.0, 1.0) > 0.0 && local < 16.5;
    let lift = if casting { 60.0 } else { 0.0 };
    let (sx, sy) = (cx + 92.0, y - 380.0 - lift);
    f.line(cx + 70.0, y - 140.0, sx, sy, 6.0, 0x3b2a1c, a, 1.0);
    f.line(
        cx + 70.0,
        y - 180.0,
        cx + 40.0,
        y - 200.0,
        4.0,
        0xc9c0b0,
        a,
        1.0,
    );
    let pulse = 0.7 + 0.3 * (t * 4.0).sin();
    f.glow(sx, sy, 60.0, FROST, 0.5 * pulse * a);
    f.disc(sx, sy, 12.0, 0xe9f6ff, a);
    // Frost: shards spiral up from the orb while casting.
    if casting {
        for i in 0..40 {
            let ph = (t * 0.8 + hash(i, 1)).fract();
            let ang = hash(i, 2) * std::f32::consts::TAU + t * 2.0;
            let r = ph * 160.0;
            let (x, yy) = (sx + ang.cos() * r, sy + ang.sin() * r * 0.6 - ph * 120.0);
            f.disc(x, yy, 2.5 * (1.0 - ph) + 1.0, 0xdff3ff, (1.0 - ph) * a);
        }
    }
}

const SLOT: f32 = 66.0;
const BAG_X: f32 = 80.0;
const BAG_Y: f32 = 1340.0;

fn slot_xy(i: usize) -> (f32, f32) {
    let (c, r) = (i % 4, i / 4);
    (
        BAG_X + 30.0 + c as f32 * SLOT + SLOT * 0.5,
        BAG_Y + 70.0 + r as f32 * SLOT + SLOT * 0.5,
    )
}

/// The bags (herbs in, potions brewed, sold to the counter) and the two
/// professions.
fn bags(f: &mut Frame, local: f32, a: f32) {
    let (x0, y0) = (BAG_X as i32, BAG_Y as i32);
    let (x1, y1) = (x0 + 30 * 2 + 4 * SLOT as i32, y0 + 90 + 3 * SLOT as i32);
    f.rect(x0, y0, x1, y1, 0x0c0d16, 0.85 * a);
    f.frame_rect(x0, y0, x1, y1, 0x8a7a52, a);
    f.text(x0 + 30, y0 + 22, "BAGS", 3, 0xe9dcb5, a);
    let sold = sold_count(local);
    let mut copper = 0u32;
    for (i, &(at, kind, price)) in HERBS.iter().enumerate() {
        let (sx, sy) = slot_xy(i);
        f.frame_rect(
            (sx - SLOT * 0.45) as i32,
            (sy - SLOT * 0.45) as i32,
            (sx + SLOT * 0.45) as i32,
            (sy + SLOT * 0.45) as i32,
            0x3a3524,
            a,
        );
        if i < sold {
            copper += price;
            continue;
        }
        if local >= at + 1.0 {
            herb(f, sx, sy + 8.0, kind, a, 0.4);
        }
    }
    for i in HERBS.len()..12 {
        let (sx, sy) = slot_xy(i);
        f.frame_rect(
            (sx - SLOT * 0.45) as i32,
            (sy - SLOT * 0.45) as i32,
            (sx + SLOT * 0.45) as i32,
            (sy + SLOT * 0.45) as i32,
            0x3a3524,
            a,
        );
    }
    for &(at, slot) in &POTIONS {
        if local >= at {
            let (sx, sy) = slot_xy(HERBS.len() + slot);
            let k = ease_out((local - at) / 0.4);
            f.disc(sx, sy + 8.0, 18.0 * k, 0x5a7dff, a);
            f.rect(
                sx as i32 - 5,
                (sy - 22.0) as i32,
                sx as i32 + 5,
                (sy - 6.0) as i32,
                0xc9c0b0,
                a * k,
            );
        }
    }
    // Coins: the price of everything sold so far.
    let (g, s, c) = (copper / 10000, (copper / 100) % 100, copper % 100);
    let coins = format!("{g}G {s:02}S {c:02}C");
    f.text(x0 + 30, y1 - 40, &coins, 2, GOLD, a);
    // Professions, beside the bags.
    let px = x1 + 40;
    for (k, (name, c)) in [("HERBALISM", 0x6fcf6f), ("ALCHEMY", 0x8fb0ff)]
        .iter()
        .enumerate()
    {
        let y = y0 + 30 + k as i32 * 120;
        f.text(px, y, name, 2, *c, a);
        let grow = ((local - 8.0) / 8.0).clamp(0.0, 1.0);
        f.rect(px, y + 34, px + 280, y + 52, 0x1c1d28, a);
        f.rect(
            px,
            y + 34,
            px + (280.0 * (0.35 + 0.25 * grow + 0.1 * k as f32)) as i32,
            y + 52,
            *c,
            0.8 * a,
        );
    }
    f.text(
        px,
        y0 + 280,
        HERB_NAMES[(local as usize) % 4],
        2,
        0xb8b0a0,
        0.7 * a,
    );
}

fn sold_count(local: f32) -> usize {
    (((local - SALES_FROM) / SALE_EVERY).floor() + 1.0).clamp(0.0, HERBS.len() as f32) as usize
}

fn chat(f: &mut Frame, local: f32, a: f32) {
    let (x0, y0, x1, y1) = (60, 400, 1020, 690);
    f.rect(x0, y0, x1, y1, 0x05060c, 0.7 * a);
    let shown: Vec<_> = CHAT.iter().filter(|(at, _, _)| local >= *at).collect();
    let skip = shown.len().saturating_sub(6);
    for (i, (at, line, c)) in shown.iter().skip(skip).enumerate() {
        let y = y0 + 20 + i as i32 * 44;
        f.text(x0 + 20, y, &typed(line, *at, 60.0, local), 2, *c, a);
    }
}
