// The film: picks the scene for the clock and paints it, then composes the
// frame in layers: the sky and the moon in pixels; the world's text grid
// over them (dissolving cell by cell from the previous scene across each
// cut); the sprites, glows, beams and lines; the front text layer (what
// passes in front of the figures); then the party: the RPG windows with
// Cordelia's and Victor's portraits acting, her energy gauge, the wordless
// dialog of pictograms, and the corner mark.
use super::stage::{Px, Stage};
use super::{campfire, crossing, dawn, falls, fireflies, forest, lake, mine, owl, village};
use crate::amiga::{self, Fire};
use crate::ascii::{Blitter, Grid, CLEAR};
use crate::fx::{ease_out, smooth, window};
use crate::gfx::{Frame, H, W};
use crate::glyphs::Glyphs;
use crate::hud::{self, Acting, Face, Portraits, Slot, WIN_H, WIN_W};
use crate::post::Look;
use crate::sprites::Cast;
use crate::timeline::{bars, now, span, Scene, SCENES};
use crate::tour::Tour;
use crate::veins::Veins;

const fn at(bar: f32, plus: f32) -> f32 {
    bars(bar) + plus
}

/// Cordelia's acting, by absolute time.
const CORDELIA: &[(f32, Face)] = &[
    (0.0, Face::Tired),
    (at(9.0, 2.4), Face::Scrutiny),
    (at(9.0, 5.0), Face::Tired),
    (at(11.0, 7.0), Face::Scrutiny),
    (at(11.0, 10.0), Face::Humor),
    (at(11.0, 12.5), Face::Relief),
    (at(17.0, 3.0), Face::Scrutiny),
    (at(17.0, 12.0), Face::Relief),
    (at(22.0, 3.0), Face::Awe),
    (at(22.0, 13.0), Face::Resolve),
    (at(29.0, 2.0), Face::Scrutiny),
    (at(29.0, mine::FRONT.1), Face::Awe),
    (at(34.0, 1.0), Face::Relief),
    (at(34.0, 6.0), Face::Awe),
    (at(38.0, 2.0), Face::Relief),
    (at(38.0, campfire::CATCH), Face::Awe),
    (at(38.0, campfire::LAUGH), Face::Laugh),
    (at(38.0, campfire::LAUGH + 5.0), Face::Humor),
    (at(38.0, campfire::LAUGH2), Face::Laugh),
    (at(38.0, campfire::STAR), Face::Awe),
    (at(38.0, campfire::STAR + 3.0), Face::Relief),
    (at(50.0, 1.0), Face::Drowsy),
    (at(50.0, 5.0), Face::Asleep),
];

const VICTOR: &[(f32, Face)] = &[
    (0.0, Face::Watch),
    (at(11.0, forest::JOIN + 1.0), Face::VRelief),
    (at(11.0, forest::GO), Face::Watch),
    (at(17.0, 3.0), Face::VResolve),
    (at(22.0, 3.0), Face::VDefiance),
    (at(22.0, 13.0), Face::VResolve),
    (at(29.0, 2.0), Face::Watch),
    (at(29.0, mine::FRONT.1), Face::VRelief),
    (at(34.0, 1.0), Face::Watch),
    (at(38.0, 2.0), Face::VRelief),
    (at(38.0, campfire::LAUGH + 0.3), Face::VLaugh),
    (at(38.0, campfire::LAUGH + 5.0), Face::VRelief),
    (at(38.0, campfire::LAUGH2 + 0.2), Face::VLaugh),
    (at(38.0, campfire::LAUGH2 + 3.0), Face::Watch),
    (at(50.0, 6.0), Face::VRelief),
];

/// Her energy through the night: low and blinking after the day's work,
/// climbing a little with each wonder, full by the fire.
const ENERGY: &[(f32, f32)] = &[
    (0.0, 0.1),
    (at(17.0, 0.0), 0.14),
    (at(22.0, 0.0), 0.26),
    (at(29.0, 0.0), 0.36),
    (at(34.0, 0.0), 0.45),
    (at(38.0, campfire::LAUGH), 0.52),
    (at(38.0, 32.0), 1.0),
];

/// The dialog's pictograms: (from, to, pictograms).
type Picto = &'static [&'static str; 12];
const DIALOG: &[(f32, f32, &[Picto])] = &[
    (at(4.0, 6.0), at(4.0, 10.0), &[&hud::MOON]),
    (
        at(11.0, forest::JOIN + 0.5),
        at(11.0, forest::JOIN + 4.0),
        &[&hud::SWORD, &hud::HEART],
    ),
    (at(17.0, 1.5), at(17.0, 5.5), &[&hud::FIREFLY]),
    (at(22.0, 1.0), at(22.0, 4.5), &[&hud::DROP]),
    (
        at(29.0, mine::FRONT.1 + 0.3),
        at(29.0, mine::FRONT.1 + 4.0),
        &[&hud::GEM],
    ),
    (at(34.0, 3.0), at(34.0, 7.0), &[&hud::STAR]),
    (at(38.0, 8.5), at(38.0, 12.0), &[&hud::CUP]),
    (at(38.0, 12.5), at(38.0, 15.5), &[&hud::PIE]),
    (at(38.0, 16.0), at(38.0, 18.8), &[&hud::MALLOW]),
    (
        at(38.0, campfire::LAUGH + 0.4),
        at(38.0, campfire::LAUGH + 4.2),
        &[&hud::HEART, &hud::HEART],
    ),
];

pub struct Demo {
    grid: Grid,
    prev: Grid,
    front: Grid,
    blitter: Blitter,
    cast: Cast,
    glyphs: Glyphs,
    portraits: Portraits,
    fire: Fire,
    tour: Tour,
    veins: Veins,
    under: Vec<Px>,
    px: Vec<Px>,
}

impl Demo {
    pub fn new() -> Self {
        let cast = Cast::load();
        let glyphs = Glyphs::render(&cast);
        Self {
            grid: Grid::new(),
            prev: Grid::new(),
            front: Grid::new(),
            blitter: Blitter::new(),
            cast,
            glyphs,
            portraits: Portraits::load(),
            fire: Fire::new(36, 22),
            tour: Tour::solve(26, (240.0, 110.0, 1680.0, 560.0), 9),
            veins: mine::veins(),
            under: Vec::new(),
            px: Vec::new(),
        }
    }

    /// Paint one scene at time t, into the world grid or (for the outgoing
    /// side of a cut) the spare one, whose pixel work is dropped.
    fn scene(&mut self, which: Scene, t: f32, into_prev: bool) -> (Look, f32) {
        let mut under = std::mem::take(&mut self.under);
        let mut px = std::mem::take(&mut self.px);
        let (mut spare_under, mut spare_px) = (Vec::new(), Vec::new());
        let grid = if into_prev {
            &mut self.prev
        } else {
            &mut self.grid
        };
        grid.clear(0);
        if !into_prev {
            self.front.clear(CLEAR);
        }
        let mut st = Stage {
            g: grid,
            front: &mut self.front,
            under: if into_prev {
                &mut spare_under
            } else {
                &mut under
            },
            px: if into_prev { &mut spare_px } else { &mut px },
            cast: &self.cast,
            glyphs: &self.glyphs,
            fire: &mut self.fire,
            tour: &self.tour,
            look: Look::default(),
            corner: 0.75,
        };
        let local = t - span(which).0;
        match which {
            Scene::Owl => owl::draw(&mut st, t, local),
            Scene::Village => village::draw(&mut st, t, local),
            Scene::Crossing => crossing::draw(&mut st, t, local),
            Scene::Forest => forest::draw(&mut st, t, local),
            Scene::Fireflies => fireflies::draw(&mut st, t, local),
            Scene::Falls => falls::draw(&mut st, t, local),
            Scene::Mine => mine::draw(&mut st, t, local, &self.veins),
            Scene::Lake => lake::draw(&mut st, t, local),
            Scene::Campfire => campfire::draw(&mut st, t, local),
            Scene::Dawn => dawn::draw(&mut st, t, local),
        }
        let out = (st.look, st.corner);
        self.under = under;
        self.px = px;
        out
    }

    pub fn frame(&mut self, f: &mut Frame, t: f32) -> Look {
        let n = now(t);
        self.under.clear();
        self.px.clear();
        // A cut dissolves over the first second of a scene.
        let idx = SCENES
            .iter()
            .position(|(s, _, _)| *s == n.scene)
            .unwrap_or(0);
        let dissolve = idx > 0 && n.local < 1.0;
        if dissolve {
            let prev = SCENES[idx - 1].0;
            self.scene(prev, span(prev).1 - 0.001, true);
        }
        let (mut look, corner) = self.scene(n.scene, t, false);
        let k_in = if dissolve { ease_out(n.local) } else { 1.0 };
        if dissolve {
            let prev = std::mem::replace(&mut self.prev, Grid::new());
            self.grid.dissolve(&prev, 1.0 - k_in, (t * 60.0) as i32);
            self.prev = prev;
        }
        if n.scene == Scene::Owl {
            look.fade *= window(n.local, 0.0, 3.0);
        }
        // Under the world: the sky and the moon (over black for the parts
        // a dissolve shows before the new sky is up).
        f.px.fill(0);
        for p in &self.under {
            match *p {
                Px::Sky { stops, n, to_y } => amiga::sky_gradient(f, &stops[..n], to_y),
                Px::Moon { x, y, r, k } => amiga::moon(f, x, y, r, k * k_in),
                _ => {}
            }
        }
        self.blitter.blit(&self.grid, f, 0, 0);
        for p in &self.px {
            match *p {
                Px::Sprite {
                    id,
                    x,
                    y,
                    light,
                    light_k,
                    rim,
                    alpha,
                    bob,
                } => self.cast.get(id).draw(
                    f,
                    x,
                    y,
                    light,
                    light_k,
                    rim,
                    alpha * smooth(k_in * 1.4),
                    bob,
                ),
                Px::Glow { x, y, r, rgb, k } => f.glow(x, y, r, rgb, k * k_in),
                Px::Line { a, b, w, rgb, k } => f.line(a, b, w, rgb, k * k_in),
                Px::Copper { y0, y1, k } => amiga::copper(
                    f,
                    t,
                    y0,
                    y1,
                    &[0xdfefff, 0x3dbb8a, 0x4a9bd1, 0xffffff],
                    4,
                    k * k_in,
                ),
                Px::Beam {
                    x,
                    y,
                    dir,
                    width,
                    len,
                    rgb,
                    k,
                } => amiga::beam(f, (x, y), dir, (width, len), rgb, k * k_in),
                Px::Mark {
                    cx,
                    cy,
                    r,
                    progress,
                    gain,
                } => {
                    demofx::logo::draw(
                        f,
                        &demofx::logo::LogoState::sized(cx, cy, r, progress, gain),
                    );
                    let a = window(progress, 0.55, 1.0) * gain;
                    let y = (cy + r + 46.0) as i32;
                    f.text_centered(cx as i32, y + 3, "SOLVERFORGE", crate::gfx::S3, 0x09131b, a);
                    f.text_centered(cx as i32, y, "SOLVERFORGE", crate::gfx::S3, 0xffe6ba, a);
                }
                Px::Sky { .. } | Px::Moon { .. } => {}
            }
        }
        self.blitter.blit(&self.front, f, 0, 0);
        self.party(f, t, n.scene, n.local);
        demofx::logo::corner(f, t, corner * look.fade);
        look
    }

    fn party(&self, f: &mut Frame, t: f32, scene: Scene, local: f32) {
        let away = match scene {
            Scene::Falls => {
                window(local, falls::OPEN + 5.0, falls::OPEN + 6.0)
                    * (1.0 - window(local, 19.0, 20.0))
            }
            Scene::Dawn => window(local, dawn::MARK_AT - 1.0, dawn::MARK_AT),
            Scene::Owl => 1.0,
            _ => 0.0,
        };
        let (tint, tint_k) = match scene {
            Scene::Campfire => (0xffc890, 0.3 * campfire::fuel(local)),
            Scene::Mine => (0xffd0a0, 0.2),
            Scene::Dawn => (0xffd8c0, 0.25 * window(local, 4.0, 14.0)),
            _ => (0xffffff, 0.0),
        };
        let y = H as i32 - WIN_H - 28;
        let her = window(t, at(4.0, 1.0), at(4.0, 2.4)) * (1.0 - away);
        let him = window(t, at(11.0, forest::JOIN), at(11.0, forest::JOIN + 1.4)) * (1.0 - away);
        let slide = |k: f32| y + ((1.0 - k) * 60.0) as i32;
        hud::draw_portrait(
            f,
            &self.portraits,
            &Acting {
                keys: CORDELIA,
                seed: 3,
            },
            Slot {
                x: 34,
                y: slide(her),
                show: her,
                tint,
                tint_k,
            },
            t,
        );
        let energy = interp(ENERGY, t);
        let filling = (interp(ENERGY, t + 0.5) - energy > 0.004) as i32 as f32;
        hud::gauge(
            f,
            (48, slide(her) + 300),
            WIN_W - 28,
            energy,
            filling,
            t,
            window(her, 0.6, 1.0),
        );
        hud::draw_portrait(
            f,
            &self.portraits,
            &Acting {
                keys: VICTOR,
                seed: 8,
            },
            Slot {
                x: W as i32 - WIN_W - 34,
                y: slide(him),
                show: him,
                tint,
                tint_k,
            },
            t,
        );
        hud::gauge(
            f,
            (W as i32 - WIN_W - 20, slide(him) + 300),
            WIN_W - 28,
            1.0,
            0.0,
            t,
            window(him, 0.6, 1.0),
        );
        for &(a, b, pics) in DIALOG {
            let show = window(t, a, a + 0.35) * (1.0 - window(t, b - 0.35, b));
            if show <= 0.0 {
                continue;
            }
            let w = 40 + pics.len() as i32 * 112;
            let (x, yy) = (W as i32 / 2 - w / 2, 36);
            hud::window_box(f, x, yy, w, 136, show, 1.0);
            if show > 0.7 {
                for (i, p) in pics.iter().enumerate() {
                    let appear = window(t, a + 0.35 + i as f32 * 0.3, a + 0.9 + i as f32 * 0.3);
                    let bob = (2.0 * (t * 3.0 + i as f32).sin()) as i32;
                    hud::pictogram(
                        f,
                        p,
                        (x + 28 + i as i32 * 112, yy + 20 + bob),
                        8,
                        0x1a1410,
                        picto_accent(p),
                        appear,
                    );
                }
            }
        }
    }
}

fn picto_accent(p: &[&str; 12]) -> u32 {
    if std::ptr::eq(p, &hud::HEART) {
        0xdc4f42
    } else if std::ptr::eq(p, &hud::CUP) {
        0x3a2010
    } else if std::ptr::eq(p, &hud::PIE) {
        0xd89a4a
    } else if std::ptr::eq(p, &hud::FIREFLY) {
        0xc8ff5a
    } else if std::ptr::eq(p, &hud::DROP) {
        0x6fb9e9
    } else if std::ptr::eq(p, &hud::SWORD) {
        0xb8c8d0
    } else if std::ptr::eq(p, &hud::GEM) {
        0x3dbb8a
    } else if std::ptr::eq(p, &hud::MALLOW) {
        0xfff4e8
    } else {
        0xf6f1df
    }
}

fn interp(keys: &[(f32, f32)], t: f32) -> f32 {
    let mut out = keys[0].1;
    for w in keys.windows(2) {
        let ((a, va), (b, vb)) = (w[0], w[1]);
        if t >= a {
            out = if t >= b {
                vb
            } else {
                va + (vb - va) * smooth((t - a) / (b - a))
            };
        }
    }
    out
}
