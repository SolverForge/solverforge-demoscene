// The Desktop takes, full frame and as recorded: nothing scales them and no
// finish sits on them. Over them the demo draws only what points at the
// story: a ring traced around what the narration names, the chapter typed
// into the Desktop's empty sidebar under the logo, and, while a search plays
// faster than it ran, how much faster. The takes come in and go out through
// the cell dissolve; where one take cuts to another, a short slice glitch
// carries the cut.
use super::{mosaic, AMBER, BLUE, EMERALD, JADE, LIME, MINT};
use crate::cut::{Kind, CUTS, RINGS, SEARCHES};
use crate::footage::Footage;
use crate::gfx::{Frame, H, W};
use crate::timeline::{hash, smooth, Now, BEAT};

const TONES: [u32; 4] = [JADE, LIME, BLUE, AMBER];
const DISSOLVE: f32 = 0.7;
const GLITCH: f32 = 0.28;

/// Returns (finish, logo gain).
pub fn draw(f: &mut Frame, footage: &mut Footage, n: Now, t: f32) -> (f32, f32) {
    footage.draw(f, t);

    if let Some(&cut) = CUTS.iter().find(|&&c| t >= c && t < c + GLITCH) {
        slices(f, (t - cut) / GLITCH, t);
    }

    for &(a, b, rect, tone) in RINGS.iter() {
        if t >= a && t < b {
            ring(f, rect, TONES[tone], t - a, b - t, t);
        }
    }

    if let Some(&(a, b, speed)) = SEARCHES.iter().find(|(a, b, _)| t >= *a && t < *b) {
        let k = smooth((t - a) / 0.4) * smooth((b - t) / 0.4);
        let spin = ['|', '/', '-', '\\'][((t * 12.0) as usize) % 4];
        f.char_at(24, 954, spin, 2, LIME, k);
        f.text(54, 954, &format!("SEARCH x{speed:.1}"), 2, MINT, k);
        // A bar that runs on the beat, the way the solve keeps working.
        let w = 260.0 * ((t - a) / (b - a));
        f.rect(24, 990, 24 + w as i32, 993, EMERALD, 0.8 * k);
    }

    // In from a scene of the demo, out to one: the cell dissolve, and the
    // finish fading with it.
    let mut finish: f32 = 0.0;
    let mut logo: f32 = 1.0;
    // The film itself opens through the dissolve, out of black.
    if n.prev().is_none_or(|k| k != Kind::Take) && n.local < DISSOLVE {
        let p = n.local / DISSOLVE;
        mosaic(f, p, (W as f32 * 0.5, H as f32 * 0.5), t);
        finish = finish.max(1.0 - smooth(p));
        logo = logo.min(smooth(p));
    }
    if n.next().is_some_and(|k| k != Kind::Take) && n.len - n.local < DISSOLVE {
        let p = (n.len - n.local) / DISSOLVE;
        mosaic(f, p, (W as f32 * 0.5, H as f32 * 0.5), t);
        finish = finish.max(1.0 - smooth(p));
        logo = logo.min(smooth(p));
    }
    (finish, logo)
}

/// A ring traced around `rect` (frame pixels), clockwise from its top-left
/// corner over 0.4 s, with brackets on its corners and a halo that breathes
/// on the beat; it fades over its last 0.3 s. Nothing is drawn inside it.
fn ring(f: &mut Frame, rect: (i32, i32, i32, i32), rgb: u32, age: f32, left: f32, t: f32) {
    let (x0, y0, x1, y1) = (rect.0 - 6, rect.1 - 6, rect.2 + 6, rect.3 + 6);
    let a = smooth(left / 0.3);
    if a <= 0.0 {
        return;
    }
    let p = smooth(age / 0.4);
    let (w, h) = ((x1 - x0) as f32, (y1 - y0) as f32);
    let per = 2.0 * (w + h);
    let mut run = p * per;
    let beat = 1.0 + 0.6 * (-(t % BEAT) * 6.0).exp();
    // The four sides in order: top, right, bottom, left.
    let sides = [
        (x0, y0, 1, 0, w),
        (x1, y0, 0, 1, h),
        (x1, y1, -1, 0, w),
        (x0, y1, 0, -1, h),
    ];
    for (sx, sy, dx, dy, len) in sides {
        let l = run.min(len);
        if l <= 0.0 {
            break;
        }
        run -= l;
        let (ex, ey) = (sx + dx * l as i32, sy + dy * l as i32);
        let (ax, bx) = (sx.min(ex), sx.max(ex));
        let (ay, by) = (sy.min(ey), sy.max(ey));
        // Halo outside, the line itself on top.
        for pad in 2..=7 {
            let k = 0.16 / (pad - 1) as f32 * beat * a;
            f.rect(ax - pad, ay - pad, bx + pad + 1, by + pad + 1, rgb, k);
        }
        f.rect(ax - 1, ay - 1, bx + 2, by + 2, rgb, 0.95 * a);
    }
    // Corner brackets, after the trace has passed them.
    let b = 22;
    let k = smooth((age - 0.3) / 0.2) * a;
    for (cx, cy, sx, sy) in [
        (x0, y0, 1, 1),
        (x1, y0, -1, 1),
        (x1, y1, -1, -1),
        (x0, y1, 1, -1),
    ] {
        let (hx0, hx1) = if sx > 0 {
            (cx - 3, cx + b)
        } else {
            (cx - b, cx + 4)
        };
        let (vy0, vy1) = if sy > 0 {
            (cy - 3, cy + b)
        } else {
            (cy - b, cy + 4)
        };
        f.rect(hx0, cy - 3, hx1, cy + 4, rgb, k);
        f.rect(cx - 3, vy0, cx + 4, vy1, rgb, k);
    }
}

/// The slice glitch over a straight cut: bands of rows thrown sideways and
/// split into red and cyan, settling over `p` 0..1.
fn slices(f: &mut Frame, p: f32, t: f32) {
    let k = (1.0 - p).powi(2);
    let frame = (t * 60.0) as u32;
    let mut row = vec![0u32; W];
    let mut y = 0;
    while y < H {
        let band = 10 + (hash(y as u32 ^ (frame * 31)) * 40.0) as usize;
        let hit = hash(y as u32 * 7 + frame) < 0.45 * k + 0.1;
        let shift = if hit {
            ((hash(y as u32 * 13 + frame * 3) - 0.5) * 260.0 * k) as i32
        } else {
            0
        };
        let split = (18.0 * k) as i32;
        for yy in y..(y + band).min(H) {
            let src = &f.px[yy * W..(yy + 1) * W];
            for x in 0..W {
                let at = |dx: i32| src[(x as i32 - shift - dx).clamp(0, W as i32 - 1) as usize];
                row[x] = if hit && split > 0 {
                    (at(split) & 0xff0000) | (at(-split) & 0x00ffff)
                } else {
                    at(0)
                };
            }
            f.px[yy * W..(yy + 1) * W].copy_from_slice(&row);
        }
        y += band;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cut::SEGMENTS;

    #[test]
    fn every_ring_sits_inside_the_frame_and_inside_a_take() {
        for &(a, b, (x0, y0, x1, y1), _) in RINGS.iter() {
            assert!(
                x0 - 13 >= 0 && y0 - 13 >= 0 && x1 + 13 <= W as i32 && y1 + 13 <= H as i32,
                "ring at {a} leaves the frame"
            );
            let seg = SEGMENTS.iter().find(|s| a >= s.0 && a < s.1).unwrap();
            assert_eq!(seg.2, Kind::Take, "ring at {a} outside a take");
            assert!(b <= seg.1 + 1e-3, "ring at {a} outlives its take");
        }
    }
}
