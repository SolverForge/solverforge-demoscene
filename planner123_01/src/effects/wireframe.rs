// ═══════════════════════════════════════════════════════════════
// 3D WIREFRAME -- Rotating objects in emerald glory
// Vectors! The classic demoscene 3D engine.
// Software rasterization. Every vertex hand-transformed.
// ═══════════════════════════════════════════════════════════════

use crate::palette::{self, EMERALD_300, EMERALD_400, EMERALD_600};
use glam::{Mat4, Vec3, Vec4};

/// A 3D wireframe object
pub struct WireObject {
    pub vertices: Vec<Vec3>,
    pub edges: Vec<(usize, usize)>,
    pub pos: Vec3,
    pub rot: Vec3,
    pub scale: f32,
}

impl WireObject {
    /// Create a cube
    pub fn cube(scale: f32) -> Self {
        let v = vec![
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
        ];
        let e = vec![
            // Back face
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            // Front face
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            // Connecting edges
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];
        Self {
            vertices: v,
            edges: e,
            pos: Vec3::ZERO,
            rot: Vec3::ZERO,
            scale,
        }
    }

    /// Create an icosahedron
    pub fn icosahedron(scale: f32) -> Self {
        let phi = (1.0 + 5.0_f32.sqrt()) / 2.0; // golden ratio
        let v = vec![
            Vec3::new(-1.0, phi, 0.0).normalize(),
            Vec3::new(1.0, phi, 0.0).normalize(),
            Vec3::new(-1.0, -phi, 0.0).normalize(),
            Vec3::new(1.0, -phi, 0.0).normalize(),
            Vec3::new(0.0, -1.0, phi).normalize(),
            Vec3::new(0.0, 1.0, phi).normalize(),
            Vec3::new(0.0, -1.0, -phi).normalize(),
            Vec3::new(0.0, 1.0, -phi).normalize(),
            Vec3::new(phi, 0.0, -1.0).normalize(),
            Vec3::new(phi, 0.0, 1.0).normalize(),
            Vec3::new(-phi, 0.0, -1.0).normalize(),
            Vec3::new(-phi, 0.0, 1.0).normalize(),
        ];
        // 30 edges of an icosahedron
        let e = vec![
            (0, 1),
            (0, 5),
            (0, 7),
            (0, 10),
            (0, 11),
            (1, 5),
            (1, 7),
            (1, 8),
            (1, 9),
            (2, 3),
            (2, 4),
            (2, 6),
            (2, 10),
            (2, 11),
            (3, 4),
            (3, 6),
            (3, 8),
            (3, 9),
            (4, 5),
            (4, 9),
            (4, 11),
            (5, 9),
            (5, 11),
            (6, 7),
            (6, 8),
            (6, 10),
            (7, 8),
            (7, 10),
            (8, 9),
            (10, 11),
        ];
        Self {
            vertices: v,
            edges: e,
            pos: Vec3::ZERO,
            rot: Vec3::ZERO,
            scale,
        }
    }

    /// Create a torus (approximated as rings of circles)
    pub fn torus(scale: f32, r_major: f32, r_minor: f32, segments: usize, rings: usize) -> Self {
        let mut vertices = Vec::new();
        let mut edges = Vec::new();

        for ring in 0..rings {
            let theta = ring as f32 * std::f32::consts::TAU / rings as f32;
            let cx = theta.cos() * r_major;
            let cy = theta.sin() * r_major;

            let ring_start = vertices.len();
            for seg in 0..segments {
                let phi = seg as f32 * std::f32::consts::TAU / segments as f32;
                let x = cx + phi.cos() * r_minor * theta.cos();
                let y = cy + phi.cos() * r_minor * theta.sin();
                let z = phi.sin() * r_minor;
                vertices.push(Vec3::new(x, y, z));

                // Edge within ring
                let next_seg = ring_start + (seg + 1) % segments;
                edges.push((ring_start + seg, next_seg));

                // Edge connecting rings
                let next_ring_start = (ring + 1) % rings * segments;
                edges.push((ring_start + seg, next_ring_start + seg));
            }
        }

        Self {
            vertices,
            edges,
            pos: Vec3::ZERO,
            rot: Vec3::ZERO,
            scale,
        }
    }

    /// Create a Swiss-design grid (flat XZ grid with vertical pillars)
    #[allow(dead_code)]
    pub fn grid_lattice(scale: f32, n: usize) -> Self {
        let mut vertices = Vec::new();
        let mut edges = Vec::new();

        // Flat grid
        for i in 0..=n {
            for j in 0..=n {
                let x = (i as f32 / n as f32) * 2.0 - 1.0;
                let z = (j as f32 / n as f32) * 2.0 - 1.0;
                vertices.push(Vec3::new(x, 0.0, z));

                // X-direction edge
                if i < n {
                    let idx = vertices.len() - 1;
                    edges.push((idx, idx + 1));
                }
                // Z-direction edge
                if j < n {
                    let idx = vertices.len() - 1;
                    edges.push((idx, idx + (n + 1)));
                }
            }
        }

        // Vertical pillars at corners
        let corner_indices = [0, n, n * (n + 1), (n + 1) * (n + 1) - 1];
        for &ci in &corner_indices {
            if ci < vertices.len() {
                let v = vertices[ci];
                let top_idx = vertices.len();
                vertices.push(Vec3::new(v.x, 1.5, v.z));
                edges.push((ci, top_idx));
            }
        }

        Self {
            vertices,
            edges,
            pos: Vec3::ZERO,
            rot: Vec3::ZERO,
            scale,
        }
    }
}

pub struct Wireframe {
    pub objects: Vec<WireObject>,
    trail_buffer: Vec<u32>,
    trail_width: usize,
    trail_height: usize,
}

impl Wireframe {
    pub fn new(width: usize, height: usize) -> Self {
        // Three objects: cube left, icosahedron center, torus right
        let mut cube = WireObject::cube(0.8);
        cube.pos = Vec3::new(-0.55, 0.0, 0.0);
        cube.rot = Vec3::new(0.3, 0.5, 0.1);

        let mut ico = WireObject::icosahedron(0.85);
        ico.pos = Vec3::new(0.0, 0.0, 0.0);
        ico.rot = Vec3::new(0.0, 0.0, 0.0);

        let mut torus = WireObject::torus(0.7, 0.6, 0.22, 12, 20);
        torus.pos = Vec3::new(0.55, 0.0, 0.0);
        torus.rot = Vec3::new(0.7, 0.2, 0.4);

        Self {
            objects: vec![cube, ico, torus],
            trail_buffer: vec![0u32; width * height],
            trail_width: width,
            trail_height: height,
        }
    }

    pub fn update(&mut self, time: f64) {
        let t = time as f32;
        // Rotate objects at different speeds (demoscene feel)
        if self.objects.len() > 0 {
            self.objects[0].rot = Vec3::new(t * 0.7, t * 1.1, t * 0.4);
        }
        if self.objects.len() > 1 {
            self.objects[1].rot = Vec3::new(t * 0.3, t * 0.8, t * 0.5);
        }
        if self.objects.len() > 2 {
            self.objects[2].rot = Vec3::new(t * 1.2, t * 0.6, t * 0.9);
        }

        // Fade trail buffer
        for px in &mut self.trail_buffer {
            let r = ((*px >> 16) & 0xFF).saturating_sub(8) as u32;
            let g = ((*px >> 8) & 0xFF).saturating_sub(10) as u32;
            let b = (*px & 0xFF).saturating_sub(5) as u32;
            *px = (r << 16) | (g << 8) | b;
        }
    }

    pub fn render(
        &mut self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        fade: f32,
        time: f64,
        show_grid: bool,
    ) {
        let t = time as f32;
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let fov = width.min(height) as f32 * 0.6;

        // Swiss grid overlay
        if show_grid {
            self.draw_grid(buffer, width, height, fade * 0.15);
        }

        // Collect all line draw commands first (avoid borrow conflict with trail_buffer)
        let mut draw_commands: Vec<(i32, i32, i32, i32, u32)> = Vec::new();

        for obj in &self.objects {
            let scale = obj.scale * fov * 0.4;

            // Build transform matrix
            let model = Mat4::from_translation(Vec3::new(
                obj.pos.x * width as f32 * 0.28,
                obj.pos.y * height as f32 * 0.28,
                0.0,
            )) * Mat4::from_rotation_x(obj.rot.x)
                * Mat4::from_rotation_y(obj.rot.y)
                * Mat4::from_rotation_z(obj.rot.z)
                * Mat4::from_scale(Vec3::splat(scale));

            // Project vertices to screen
            let screen_verts: Vec<Option<(i32, i32, f32)>> = obj
                .vertices
                .iter()
                .map(|v| {
                    let v4 = model * Vec4::new(v.x, v.y, v.z, 1.0);
                    let z = v4.z + fov * 1.8; // camera distance
                    if z <= 0.1 {
                        return None;
                    }
                    let proj_x = (cx + v4.x * fov / z) as i32;
                    let proj_y = (cy - v4.y * fov / z) as i32;
                    Some((proj_x, proj_y, z))
                })
                .collect();

            // Collect draw commands
            for &(i, j) in &obj.edges {
                if i >= screen_verts.len() || j >= screen_verts.len() {
                    continue;
                }
                let (Some((x0, y0, z0)), Some((x1, y1, z1))) = (screen_verts[i], screen_verts[j])
                else {
                    continue;
                };

                let avg_z = (z0 + z1) / 2.0;
                let depth_t = (fov * 2.0 / avg_z).clamp(0.0, 1.0);
                let edge_color = palette::lerp_color(
                    palette::dim(EMERALD_600, depth_t * 0.5 * fade),
                    palette::dim(EMERALD_300, depth_t * fade),
                    depth_t,
                );
                draw_commands.push((x0, y0, x1, y1, edge_color));
            }
        }

        // Execute draw commands (now we can mutably borrow trail_buffer)
        for &(x0, y0, x1, y1, color) in &draw_commands {
            self.draw_line_to_trail(x0, y0, x1, y1, color, width, height);
            Self::draw_line_static(buffer, width, height, x0, y0, x1, y1, color);
        }

        // Composite trail onto buffer
        if self.trail_width == width && self.trail_height == height {
            for i in 0..buffer.len() {
                buffer[i] = palette::add_color(buffer[i], self.trail_buffer[i]);
            }
        }

        // Label objects (Swiss design typography)
        let labels = ["CUBE", "ICOSAHEDRON", "TORUS"];
        let label_positions = [
            (
                cx as i32 - (width as i32 / 4),
                cy as i32 + (height as i32 / 3),
            ),
            (cx as i32, cy as i32 + (height as i32 / 3)),
            (
                cx as i32 + (width as i32 / 4),
                cy as i32 + (height as i32 / 3),
            ),
        ];
        for (i, (label, (lx, ly))) in labels.iter().zip(label_positions.iter()).enumerate() {
            let flash = ((t * 3.0 + i as f32 * 2.1).sin() * 0.1 + 0.9).abs();
            let lc = palette::dim(EMERALD_400, fade * 0.6 * flash);
            crate::font::draw_text_centered(buffer, width, height, label, *lx, *ly, 1, lc);
        }
    }

    fn draw_grid(&self, buffer: &mut [u32], width: usize, height: usize, alpha: f32) {
        let grid_color = palette::dim(EMERALD_600, alpha);
        let cols = 12;
        let rows = 8;

        // Vertical lines (12-column Swiss grid)
        for i in 0..=cols {
            let x = (i * width / cols) as i32;
            for y in 0..height as i32 {
                if x >= 0 && x < width as i32 {
                    buffer[y as usize * width + x as usize] =
                        palette::add_color(buffer[y as usize * width + x as usize], grid_color);
                }
            }
        }

        // Horizontal lines
        for i in 0..=rows {
            let y = (i * height / rows) as i32;
            for x in 0..width as i32 {
                if y >= 0 && y < height as i32 {
                    buffer[y as usize * width + x as usize] =
                        palette::add_color(buffer[y as usize * width + x as usize], grid_color);
                }
            }
        }
    }

    fn draw_line_to_trail(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: u32,
        width: usize,
        height: usize,
    ) {
        if self.trail_width != width || self.trail_height != height {
            return;
        }
        let trail = &mut self.trail_buffer;
        bresenham(x0, y0, x1, y1, width, height, |px, py| {
            let idx = py * width + px;
            trail[idx] = palette::add_color(trail[idx], palette::dim(color, 0.6));
        });
    }

    fn draw_line_static(
        buffer: &mut [u32],
        width: usize,
        height: usize,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: u32,
    ) {
        bresenham(x0, y0, x1, y1, width, height, |px, py| {
            let idx = py * width + px;
            buffer[idx] = palette::add_color(buffer[idx], color);
        });
    }
}

/// Bresenham line algorithm
fn bresenham<F: FnMut(usize, usize)>(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    width: usize,
    height: usize,
    mut plot: F,
) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1i32 } else { -1 };
    let sy = if y0 < y1 { 1i32 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
            plot(x as usize, y as usize);
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            if x == x1 {
                break;
            }
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            if y == y1 {
                break;
            }
            err += dx;
            y += sy;
        }
    }
}
