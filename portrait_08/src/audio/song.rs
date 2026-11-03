// PORTRAIT_08 soundtrack: a guitar instrumental at 120 BPM in E harmonic
// minor. Every guitar note comes from score.rs -- the same list the guitar
// on screen rings its strings from -- and is synthesized as a plucked
// string (Karplus-Strong): a burst of noise circulating in a delay line one
// period long. Bends shorten the delay line while the note rings, so the
// pitch slides for real. Clean notes go straight to the mix; power chords,
// palm mutes and the lead go through an amp: drive, soft clipping and a
// speaker-cabinet filter. Under it: a picked bass, a kit (kick, gated
// snare, hats, toms, crash), and a soft pad.
//
// Everything is rendered once into buffers at startup; live playback and
// the WAV export read the same samples.
use crate::score::{self, Kind, Note, BASS};
use crate::timeline::{bars, now, span, Section, BAR, BEAT, DURATION, TOTAL_BARS};

pub const SAMPLE_RATE: u32 = 44_100;
const SR: f32 = SAMPLE_RATE as f32;
const TAU: f32 = std::f32::consts::TAU;

fn hz(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

fn coeff(cutoff_hz: f32) -> f32 {
    1.0 - (-TAU * cutoff_hz / SR).exp()
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

struct Mix {
    /// Clean guitar, bass and drums, stereo.
    dry: Vec<[f32; 2]>,
    /// Pad, ducked under the kick at the end.
    pad: Vec<[f32; 2]>,
    /// Rhythm guitar into the amp.
    rhythm: Vec<f32>,
    /// Lead guitar into its own amp.
    lead: Vec<f32>,
    /// Into the delay.
    send: Vec<f32>,
    kicks: Vec<f32>,
    noise: u64,
}

impl Mix {
    fn new(len: usize) -> Self {
        Self {
            dry: vec![[0.0; 2]; len],
            pad: vec![[0.0; 2]; len],
            rhythm: vec![0.0; len],
            lead: vec![0.0; len],
            send: vec![0.0; len],
            kicks: Vec::new(),
            noise: 0x1234_5678_9abc_def1,
        }
    }

    fn rnd(&mut self) -> f32 {
        self.noise ^= self.noise << 13;
        self.noise ^= self.noise >> 7;
        self.noise ^= self.noise << 17;
        (self.noise >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0
    }

    /// A stereo voice into `dry`: `f(t)` gives the mono sample t seconds in.
    #[allow(clippy::too_many_arguments)]
    fn voice(
        &mut self,
        at: f32,
        dur: f32,
        gain: f32,
        pan: f32,
        send: f32,
        mut f: impl FnMut(f32, &mut Self) -> f32,
    ) {
        let i0 = (at * SR) as usize;
        let n = (dur * SR) as usize;
        let (gl, gr) = (
            (1.0 - pan) * 0.5 * gain * 1.4,
            (1.0 + pan) * 0.5 * gain * 1.4,
        );
        for k in 0..n {
            let i = i0 + k;
            if i >= self.dry.len() {
                break;
            }
            let s = f(k as f32 / SR, self);
            self.dry[i][0] += s * gl;
            self.dry[i][1] += s * gr;
            self.send[i] += s * send * gain;
        }
    }
}

#[derive(Clone, Copy)]
enum Bus {
    Clean(f32),
    Rhythm,
    Lead,
}

/// One plucked string. `pitch(t)` is the MIDI pitch t seconds in, so a
/// bend is just a pitch curve; `bright` sets the pick's attack, `loss` how
/// fast the string dies, `lp` how much each pass dulls it.
#[allow(clippy::too_many_arguments)]
fn pluck(
    m: &mut Mix,
    bus: Bus,
    at: f32,
    dur: f32,
    vel: f32,
    bright: f32,
    loss: f32,
    lp: f32,
    pitch: impl Fn(f32) -> f32,
) {
    const LEN: usize = 4096;
    let mut line = vec![0.0f32; LEN];
    let n0 = (SR / hz(pitch(0.0))).min(LEN as f32 - 2.0);
    // Excite: a burst of filtered noise one period long, DC removed.
    let mut f = 0.0f32;
    let mut sum = 0.0f32;
    let cnt = n0.ceil() as usize;
    for v in line.iter_mut().take(cnt) {
        f += (m.rnd() - f) * bright;
        *v = f;
        sum += f;
    }
    let mean = sum / cnt as f32;
    for v in line.iter_mut().take(cnt) {
        *v -= mean;
    }
    let mut prev = 0.0f32;
    let i0 = (at * SR) as usize;
    let release = 0.08;
    let total = ((dur + release) * SR) as usize;
    for (k, w) in (0..total).zip(cnt..) {
        let i = i0 + k;
        if i >= m.dry.len() {
            break;
        }
        let t = k as f32 / SR;
        let n = (SR / hz(pitch(t))).clamp(2.0, LEN as f32 - 2.0);
        let r = w as f32 - n;
        let r = if r < 0.0 { r + LEN as f32 } else { r };
        let (ri, frac) = (r as usize % LEN, r.fract());
        let y = line[ri] + (line[(ri + 1) % LEN] - line[ri]) * frac;
        let gate = if t < dur {
            1.0
        } else {
            (1.0 - (t - dur) / release).max(0.0)
        };
        let next = loss * (lp * y + (1.0 - lp) * prev) * (0.985 + 0.015 * gate);
        prev = y;
        line[w % LEN] = next;
        let s = y * vel * gate;
        match bus {
            Bus::Clean(pan) => {
                m.dry[i][0] += s * (1.0 - pan) * 0.5;
                m.dry[i][1] += s * (1.0 + pan) * 0.5;
                m.send[i] += s * 0.25;
            }
            Bus::Rhythm => m.rhythm[i] += s,
            Bus::Lead => m.lead[i] += s,
        }
    }
}

fn guitar(m: &mut Mix, n: &Note) {
    let base = n.midi;
    match n.kind {
        Kind::Clean => {
            let pan = (n.string as f32 - 2.5) * 0.18;
            pluck(
                m,
                Bus::Clean(pan),
                n.at,
                n.dur,
                0.34 * n.vel,
                0.55,
                0.9965,
                0.5,
                move |_| base,
            );
        }
        Kind::Power => pluck(
            m,
            Bus::Rhythm,
            n.at,
            n.dur,
            0.32 * n.vel,
            0.7,
            0.998,
            0.5,
            move |_| base,
        ),
        Kind::Mute => pluck(
            m,
            Bus::Rhythm,
            n.at,
            n.dur,
            0.36 * n.vel,
            0.35,
            0.986,
            0.32,
            move |_| base,
        ),
        Kind::Lead => {
            let (from, dur) = (n.bend_from.unwrap_or(base), n.dur);
            pluck(
                m,
                Bus::Lead,
                n.at,
                n.dur,
                0.3 * n.vel,
                0.7,
                0.9985,
                0.5,
                move |t| {
                    // Bend up over a sixteenth; long notes grow a vibrato.
                    let bent = from + (base - from) * smooth(t / 0.13);
                    let vib = 0.28
                        * (TAU * 5.6 * t).sin()
                        * smooth((t - 0.3) / 0.3)
                        * (dur > 0.6) as i32 as f32;
                    bent + vib
                },
            );
        }
    }
}

pub fn render() -> Vec<[f32; 2]> {
    let len = (DURATION * SR) as usize + 1;
    let mut m = Mix::new(len);

    for n in score::score() {
        guitar(&mut m, &n);
    }

    for b in 0..TOTAL_BARS as usize {
        let t0 = bars(b as f32);
        let sec = now(t0 + 0.01).section;
        let next = now(t0 + BAR + 0.01).section;
        let (s0, s1) = span(sec);
        let k = b - s0;
        let c = score::chord(b);
        let entering = k == 0 && b > 0;
        match sec {
            Section::Guitar => {
                if b >= 4 {
                    bass(&mut m, t0, c, false, 0.7);
                }
            }
            Section::Worlds => {
                pad(&mut m, t0, c, 0.8);
                bass(&mut m, t0, c, false, 0.8);
                if k >= 4 {
                    kick(&mut m, t0, 0.8);
                    snare(&mut m, t0 + 2.0 * BEAT, 0.8);
                    hats(&mut m, t0, 2, 0.8);
                    if k == 4 {
                        crash(&mut m, t0);
                    }
                }
            }
            Section::Challenge if k < 8 => {
                if entering {
                    crash(&mut m, t0);
                }
                bass(&mut m, t0, c, true, 1.0);
                for e in 0..8 {
                    kick(
                        &mut m,
                        t0 + e as f32 * BEAT / 2.0,
                        if e % 2 == 0 { 1.0 } else { 0.7 },
                    );
                }
                snare(&mut m, t0 + BEAT, 1.0);
                snare(&mut m, t0 + 3.0 * BEAT, 1.0);
            }
            Section::Challenge => {
                // The hall: a slow heartbeat and a clock.
                for beat in [0.0, 2.0] {
                    kick(&mut m, t0 + beat * BEAT, 0.45);
                    kick(&mut m, t0 + beat * BEAT + 0.28, 0.3);
                }
                for beat in 0..4 {
                    tick(&mut m, t0 + beat as f32 * BEAT);
                }
                pad(&mut m, t0, c, 0.5);
            }
            Section::Machine | Section::Work => {
                if entering {
                    crash(&mut m, t0);
                }
                bass(&mut m, t0, c, true, 1.0);
                for beat in [0.0, 1.5, 2.0] {
                    kick(&mut m, t0 + beat * BEAT, 1.0);
                }
                snare(&mut m, t0 + BEAT, 1.0);
                snare(&mut m, t0 + 3.0 * BEAT, 1.0);
                hats(&mut m, t0, 2, 1.0);
                if k % 4 == 0 && !entering {
                    crash(&mut m, t0);
                }
            }
            Section::Reading => {
                pad(&mut m, t0, c, 0.7);
                bass(&mut m, t0, c, false, 0.8);
                kick(&mut m, t0, 0.8);
                snare(&mut m, t0 + 2.0 * BEAT, 0.7);
                hats(&mut m, t0, 2, 0.6);
            }
            Section::Home => {
                pad(&mut m, t0, c, 0.9);
                bass(&mut m, t0, c, false, 0.7);
                kick(&mut m, t0, 0.5);
                hats(&mut m, t0, 1, 0.6);
            }
            Section::Name => {
                if k == 0 {
                    crash(&mut m, t0);
                    kick(&mut m, t0, 1.0);
                    bass(&mut m, t0, 0, false, 1.0);
                }
                if k >= 3 {
                    pad(&mut m, t0, 0, 0.7);
                }
            }
        }
        // A tom fill into the loud sections.
        if next != sec && matches!(next, Section::Challenge | Section::Machine | Section::Work) {
            toms(&mut m, t0);
        }
        let _ = s1;
    }

    mixdown(m)
}

/// The bass, an octave under the guitar: eighths or one long note.
fn bass(m: &mut Mix, t0: f32, c: usize, eighths: bool, vel: f32) {
    let root = BASS[c];
    let n = if eighths { 8 } else { 1 };
    let len = if eighths {
        BEAT / 2.0 * 0.85
    } else {
        BAR * 0.97
    };
    for e in 0..n {
        let up = if eighths && e == 7 { 7.0 } else { 0.0 };
        let at = t0 + e as f32 * BEAT / 2.0;
        let r = root + up;
        pluck(
            m,
            Bus::Clean(0.0),
            at,
            len,
            0.55 * vel,
            0.3,
            0.997,
            0.45,
            move |_| r,
        );
    }
}

/// A soft pad: three triangle voices per chord tone, slow attack.
fn pad(m: &mut Mix, t0: f32, c: usize, gain: f32) {
    const PADS: [[f32; 3]; 4] = [
        [52.0, 55.0, 59.0],
        [48.0, 52.0, 55.0],
        [45.0, 48.0, 52.0],
        [47.0, 51.0, 54.0],
    ];
    let dur = BAR + 0.4;
    for (k, &note) in PADS[c].iter().enumerate() {
        let f = hz(note + 12.0);
        let i0 = (t0 * SR) as usize;
        let pan = (k as f32 - 1.0) * 0.5;
        for s in 0..(dur * SR) as usize {
            let i = i0 + s;
            if i >= m.pad.len() {
                break;
            }
            let t = s as f32 / SR;
            let ph = (f * t + k as f32 * 0.3).fract();
            let tri = 1.0 - 4.0 * (ph - 0.5).abs();
            let e = smooth(t / 0.5) * smooth((dur - t) / 0.5);
            let v = tri * e * 0.03 * gain;
            m.pad[i][0] += v * (1.0 - pan) * 0.5 * 1.4;
            m.pad[i][1] += v * (1.0 + pan) * 0.5 * 1.4;
        }
    }
}

fn kick(m: &mut Mix, at: f32, gain: f32) {
    m.kicks.push(at);
    let mut phase = 0.0f32;
    m.voice(at, 0.4, 0.5 * gain, 0.0, 0.0, move |t, _| {
        let f = 50.0 + 120.0 * (-t * 32.0).exp();
        phase += f / SR;
        (phase * TAU).sin() * (-t * 7.5).exp() + 0.35 * (-t * 280.0).exp()
    });
}

fn snare(m: &mut Mix, at: f32, gain: f32) {
    let mut lp = 0.0f32;
    m.voice(at, 0.3, gain * 0.3, 0.05, 0.15, move |t, mm| {
        let n = mm.rnd();
        lp += (n - lp) * 0.45;
        let crack = 0.8 * n * (-t * 25.0).exp() + 0.6 * (TAU * 190.0 * t).sin() * (-t * 20.0).exp();
        crack + 0.35 * lp * (-t * 9.0).exp()
    });
}

fn hats(m: &mut Mix, t0: f32, per_beat: usize, gain: f32) {
    for k in 0..4 * per_beat {
        let at = t0 + k as f32 * BEAT / per_beat as f32;
        let accent = if k % 2 == 0 { 1.0 } else { 0.65 };
        let mut prev = 0.0f32;
        m.voice(at, 0.06, 0.07 * accent * gain, 0.3, 0.0, move |t, mm| {
            let n = mm.rnd();
            let hp = n - prev;
            prev = n;
            hp * (-t * 65.0).exp()
        });
    }
}

fn crash(m: &mut Mix, at: f32) {
    let mut prev = 0.0f32;
    m.voice(at, 2.4, 0.16, -0.25, 0.1, move |t, mm| {
        let n = mm.rnd();
        let hp = n - prev;
        prev = n * 0.6;
        hp * (-t * 1.8).exp()
    });
}

/// A clock in the hall: a short, high click.
fn tick(m: &mut Mix, at: f32) {
    m.voice(at, 0.04, 0.05, 0.4, 0.3, move |t, _| {
        (TAU * 2800.0 * t).sin() * (-t * 120.0).exp()
    });
}

/// Six toms down the kit over the last two beats of a bar.
fn toms(m: &mut Mix, t0: f32) {
    for k in 0..6 {
        let at = t0 + 2.0 * BEAT + k as f32 * BEAT / 3.0;
        let base = 200.0 - k as f32 * 22.0;
        let pan = 0.6 - k as f32 * 0.24;
        let mut phase = 0.0f32;
        m.voice(at, 0.4, 0.28, pan, 0.15, move |t, mm| {
            let f = base * (1.0 + 0.6 * (-t * 22.0).exp());
            phase += f / SR;
            (phase * TAU).sin() * (-t * 7.0).exp() + 0.15 * mm.rnd() * (-t * 40.0).exp()
        });
    }
}

/// Drive, soft clipping and a cabinet: a high-pass to tighten the low end,
/// asymmetric tanh, then two low-poles for the speaker.
fn amp(input: &[f32], drive: f32, cab_hz: f32) -> Vec<f32> {
    let (k_hp, k_cab) = (coeff(110.0), coeff(cab_hz));
    let (mut hp, mut a, mut b) = (0.0f32, 0.0f32, 0.0f32);
    let bias = 0.12f32;
    input
        .iter()
        .map(|&x| {
            hp += (x - hp) * k_hp;
            let y = (drive * (x - hp) + bias).tanh() - bias.tanh();
            a += (y - a) * k_cab;
            b += (a - b) * k_cab;
            b
        })
        .collect()
}

fn mixdown(m: Mix) -> Vec<[f32; 2]> {
    let len = m.dry.len();
    let mut out = m.dry;

    // Rhythm guitar: one amp, the right side 14 ms late for width.
    let rhythm = amp(&m.rhythm, 9.0, 3600.0);
    let haas = (0.014 * SR) as usize;
    for i in 0..len {
        let r = if i >= haas { rhythm[i - haas] } else { 0.0 };
        out[i][0] += 0.36 * rhythm[i];
        out[i][1] += 0.36 * r;
    }
    // Lead: hotter amp, a touch left of centre, into the delay.
    let lead = amp(&m.lead, 14.0, 4200.0);
    let mut send = m.send;
    for i in 0..len {
        out[i][0] += 0.34 * lead[i];
        out[i][1] += 0.26 * lead[i];
        send[i] += 0.45 * lead[i];
    }

    // The pad ducks under every kick.
    let mut kicks = m.kicks;
    kicks.sort_by(f32::total_cmp);
    let mut ki = 0;
    for (i, frame) in out.iter_mut().enumerate() {
        let t = i as f32 / SR;
        while ki + 1 < kicks.len() && kicks[ki + 1] <= t {
            ki += 1;
        }
        let duck = match kicks.get(ki) {
            Some(&k) if k <= t => 1.0 - 0.45 * (-(t - k) * 8.0).exp(),
            _ => 1.0,
        };
        frame[0] += m.pad[i][0] * duck;
        frame[1] += m.pad[i][1] * duck;
    }

    // Ping-pong delay: a dotted eighth.
    let d = (BEAT * 0.75 * SR) as usize;
    let mut l = vec![0.0f32; len];
    let mut r = vec![0.0f32; len];
    for i in 0..len {
        let fb_l = if i >= d { r[i - d] * 0.38 } else { 0.0 };
        let fb_r = if i >= d { l[i - d] * 0.38 } else { 0.0 };
        l[i] = send[i] + fb_l;
        r[i] = fb_r;
    }
    for i in 0..len {
        out[i][0] += 0.28 * l[i];
        out[i][1] += 0.28 * r[i];
    }

    // Soft clip, normalise to a fixed peak, fade the ends.
    let mut peak = 0.0f32;
    for frame in out.iter_mut() {
        for s in frame.iter_mut() {
            *s = (*s * 1.1).tanh();
            peak = peak.max(s.abs());
        }
    }
    let g = if peak > 0.0 { 0.89 / peak } else { 1.0 };
    for (i, frame) in out.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let fade = (t / 0.2).min(1.0) * ((DURATION - t) / 2.0).clamp(0.0, 1.0);
        frame[0] *= g * fade;
        frame[1] *= g * fade;
    }
    out
}
