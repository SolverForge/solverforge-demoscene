// Small signed-distance helpers for the drawn objects (the guitar): negative inside, in the object's own units.

pub fn circle(p: (f32, f32), c: (f32, f32), r: f32) -> f32 {
    ((p.0 - c.0).powi(2) + (p.1 - c.1).powi(2)).sqrt() - r
}

/// An ellipse, approximated by scaling space (good enough near the edge).
pub fn ellipse(p: (f32, f32), c: (f32, f32), r: (f32, f32)) -> f32 {
    let (dx, dy) = ((p.0 - c.0) / r.0, (p.1 - c.1) / r.1);
    ((dx * dx + dy * dy).sqrt() - 1.0) * r.0.min(r.1)
}

/// A box with rounded corners, centre c, half sizes h, corner radius k.
pub fn rbox(p: (f32, f32), c: (f32, f32), h: (f32, f32), k: f32) -> f32 {
    let qx = (p.0 - c.0).abs() - h.0 + k;
    let qy = (p.1 - c.1).abs() - h.1 + k;
    (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - k
}

pub fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

pub fn smax(a: f32, b: f32, k: f32) -> f32 {
    -smin(-a, -b, k)
}

/// Coverage of a shape at distance d for a pixel `px` units wide.
pub fn cover(d: f32, px: f32) -> f32 {
    (0.5 - d / px).clamp(0.0, 1.0)
}
