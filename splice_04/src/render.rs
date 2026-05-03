use crate::framebuffer::Framebuffer;
use crate::math::{ease_in_out, lerp, smoothstep, Vec2};
use crate::palette::{
    mix, Color, EMERALD, EMERALD_DARK, GLASS, GOLD, MINT, PANIC, STONE, STONE_DARK, VOID,
};
use crate::route::RouteState;
use crate::route_plan::{RoutePlan, StationRole};
use crate::scene::{scene_state, SceneId, SceneState};

pub fn render_frame(fb: &mut Framebuffer, plan: &RoutePlan, route: &RouteState, t: f32) {
    let scene = scene_state(t);
    draw_background(fb, scene, t);
    draw_plan_floor(fb, plan, scene);
    draw_floor_edges(fb, plan, scene);

    match scene.id {
        SceneId::Invocation => {}
        SceneId::Stations => draw_route(fb, plan, route.initial_route(), scene.progress, t, false),
        SceneId::Successors => {
            draw_route(fb, plan, route.initial_route(), 1.0, t, true);
            draw_successor_labels(fb, plan, route.initial_route(), t);
        }
        SceneId::RequiredStation => {
            draw_route(fb, plan, route.initial_route(), 1.0, t, true);
            draw_required_station(fb, plan, scene.progress, t);
            draw_candidate_arcs(fb, plan, route, scene.progress * 0.7, None);
        }
        SceneId::InsertionSearch => {
            draw_route(fb, plan, route.initial_route(), 1.0, t, true);
            draw_required_station(fb, plan, 1.0, t);
            draw_candidate_arcs(fb, plan, route, 1.0, Some(scene.progress));
            draw_text(fb, 86, 642, "TRY INSERT AT POS 4", GOLD, 2, 0.88);
        }
        SceneId::Splice => draw_splice(fb, plan, route, scene, t),
        SceneId::ConstraintRepair => {
            draw_route(fb, plan, route.inserted_route(), 1.0, t, true);
            draw_constraint_badges(fb, plan, route, scene.progress, t);
        }
        SceneId::Reroute => draw_reroute(fb, plan, route, scene, t),
        SceneId::FinalRoute | SceneId::Scrolltext => {
            draw_route(fb, plan, route.rerouted_route(), 1.0, t, true);
            draw_final_lock(fb, plan, scene, t);
        }
    }

    draw_station_beads(fb, plan, route, scene, t);
    draw_captions(fb, scene);
    if matches!(scene.id, SceneId::Scrolltext) {
        draw_scrolltext(fb, scene);
    }
    draw_post(fb);
}

fn draw_background(fb: &mut Framebuffer, scene: SceneState, t: f32) {
    let top = VOID;
    let bottom = if matches!(scene.id, SceneId::FinalRoute | SceneId::Scrolltext) {
        Color::new(28, 24, 12)
    } else {
        Color::new(8, 15, 16)
    };

    for y in 0..fb.height as i32 {
        let yf = y as f32 / fb.height as f32;
        let row = mix(top, bottom, yf);
        for x in 0..fb.width as i32 {
            fb.put_pixel(x, y, row);
        }
    }

    let breathe = (t * 0.45).sin() * 0.5 + 0.5;
    draw_radial_glow(fb, 640.0, 476.0, 480.0, EMERALD, 0.08 + breathe * 0.035);
    draw_radial_glow(fb, 790.0, 420.0, 260.0, GOLD, 0.06 + scene.progress * 0.03);

    for i in 0..95 {
        let seed = i as f32 * 21.17;
        let x = ((seed.sin() * 43837.0).abs().fract() * fb.width as f32) as i32;
        let y = (((seed * 0.61).cos() * 9187.0).abs().fract() * fb.height as f32 * 0.52) as i32;
        let twinkle = ((t * 0.35 + i as f32 * 0.17).sin() * 0.5 + 0.5) * 0.22 + 0.03;
        fb.add_pixel(x, y, if i % 9 == 0 { GOLD } else { GLASS }, twinkle);
    }
}

fn draw_plan_floor(fb: &mut Framebuffer, plan: &RoutePlan, scene: SceneState) {
    let reveal = if matches!(scene.id, SceneId::Invocation) {
        ease_in_out(scene.progress)
    } else {
        1.0
    };
    if reveal <= 0.0 {
        return;
    }

    draw_filled_rect(fb, 132, 374, 748, 294, STONE_DARK, 0.58 * reveal);
    draw_filled_rect(
        fb,
        246,
        414,
        744,
        196,
        Color::new(10, 17, 18),
        0.64 * reveal,
    );
    draw_filled_rect(
        fb,
        392,
        336,
        460,
        338,
        Color::new(11, 18, 19),
        0.50 * reveal,
    );
    draw_filled_rect(
        fb,
        742,
        360,
        374,
        250,
        Color::new(10, 16, 17),
        0.58 * reveal,
    );

    for radius in [106, 138, 172] {
        draw_ring_soft(
            fb,
            928,
            514,
            radius,
            mix(STONE, EMERALD_DARK, 0.34),
            0.18 * reveal,
        );
    }
    draw_ring_soft(fb, 790, 420, 58, mix(GOLD, STONE, 0.36), 0.32 * reveal);

    for p in &plan.guide_axis {
        draw_disc_soft(fb, p.x as i32, p.y as i32, 3, EMERALD_DARK, 0.28 * reveal);
    }

    for x in [220, 342, 464, 586, 708, 830] {
        draw_line_soft(fb, (x, 392), (x + 86, 656), EMERALD_DARK, 0.18 * reveal, 1);
        draw_line_soft(
            fb,
            (x + 70, 392),
            (x - 34, 656),
            EMERALD_DARK,
            0.12 * reveal,
            1,
        );
    }
}

fn draw_floor_edges(fb: &mut Framebuffer, plan: &RoutePlan, scene: SceneState) {
    let reveal = if matches!(scene.id, SceneId::Invocation) {
        smoothstep((scene.local_t - 2.0) / 7.0)
    } else {
        1.0
    };
    for edge in &plan.floor_edges {
        let a = plan.station_position(edge.a);
        let b = plan.station_position(edge.b);
        draw_line_soft(
            fb,
            (a.x as i32, a.y as i32),
            (b.x as i32, b.y as i32),
            mix(STONE, EMERALD_DARK, 0.45),
            0.22 * reveal,
            1,
        );
    }
}

fn draw_route(
    fb: &mut Framebuffer,
    plan: &RoutePlan,
    route: &[usize],
    reveal: f32,
    t: f32,
    arrows: bool,
) {
    let max_edge = reveal * (route.len().saturating_sub(1)) as f32;
    for (idx, pair) in route.windows(2).enumerate() {
        let partial = (max_edge - idx as f32).clamp(0.0, 1.0);
        if partial <= 0.0 {
            continue;
        }
        let a = plan.station_position(pair[0]);
        let b = plan.station_position(pair[1]);
        let end = Vec2::new(lerp(a.x, b.x, partial), lerp(a.y, b.y, partial));
        draw_line_soft(
            fb,
            (a.x as i32, a.y as i32),
            (end.x as i32, end.y as i32),
            GOLD,
            0.58,
            3,
        );
        draw_line_soft(
            fb,
            (a.x as i32, a.y as i32),
            (end.x as i32, end.y as i32),
            MINT,
            0.18,
            1,
        );
        if arrows && partial >= 1.0 {
            draw_moving_arrow(fb, a, b, t + idx as f32 * 0.13);
        }
    }
}

fn draw_station_beads(
    fb: &mut Framebuffer,
    plan: &RoutePlan,
    route: &RouteState,
    scene: SceneState,
    t: f32,
) {
    let active_route = route.visible_route(scene.id);
    let reveal_count = if matches!(scene.id, SceneId::Stations) {
        (scene.progress * active_route.len() as f32 + 0.8).floor() as usize
    } else if matches!(scene.id, SceneId::Invocation) {
        0
    } else {
        active_route.len()
    };

    for (idx, &station_idx) in active_route.iter().enumerate() {
        if idx >= reveal_count {
            continue;
        }
        let p = plan.station_position(station_idx);
        let downstream = matches!(scene.id, SceneId::Splice | SceneId::ConstraintRepair)
            && idx >= route.affected_start;
        let lift = if downstream {
            ((t * 8.0 - idx as f32 * 0.7).sin() * 0.5 + 0.5) * 5.0
        } else {
            0.0
        };
        let color = if station_idx == plan.required_station {
            GLASS
        } else {
            role_color(plan.stations[station_idx].role)
        };
        draw_disc_soft(fb, p.x as i32, p.y as i32, 16, color, 0.26);
        draw_disc_soft(fb, p.x as i32, p.y as i32, 8, mix(color, GOLD, 0.38), 0.84);
        if matches!(
            scene.id,
            SceneId::Stations | SceneId::Successors | SceneId::FinalRoute | SceneId::Scrolltext
        ) {
            draw_text(
                fb,
                p.x as i32 - 18,
                p.y as i32 + 16,
                plan.stations[station_idx].label,
                role_color(plan.stations[station_idx].role),
                1,
                0.34,
            );
        }
        draw_text(
            fb,
            p.x as i32 - 5,
            p.y as i32 - 27 - lift as i32,
            &idx.to_string(),
            MINT,
            1,
            0.94,
        );
    }

    if !matches!(scene.id, SceneId::Invocation | SceneId::Stations) {
        for (idx, station) in plan.stations.iter().enumerate() {
            if active_route.contains(&idx) {
                continue;
            }
            draw_disc_soft(
                fb,
                station.x as i32,
                station.y as i32,
                5,
                role_color(station.role),
                0.35,
            );
        }
    }
}

fn draw_successor_labels(fb: &mut Framebuffer, plan: &RoutePlan, route: &[usize], t: f32) {
    for (idx, pair) in route.windows(2).enumerate() {
        if idx % 2 == 1 {
            continue;
        }
        let a = plan.station_position(pair[0]);
        let b = plan.station_position(pair[1]);
        let x = ((a.x + b.x) * 0.5) as i32 - 12;
        let y = ((a.y + b.y) * 0.5) as i32 - 24;
        let pulse = ((t * 3.0 + idx as f32).sin() * 0.5 + 0.5) * 0.24 + 0.52;
        draw_text(fb, x, y, &format!("{idx}>{}", idx + 1), GLASS, 1, pulse);
    }
}

fn draw_required_station(fb: &mut Framebuffer, plan: &RoutePlan, reveal: f32, t: f32) {
    let station = plan.required_station;
    let p = plan.station_position(station);
    let pulse = ((t * 4.8).sin() * 0.5 + 0.5) * 8.0;
    draw_radial_glow(fb, p.x, p.y, 95.0, GLASS, 0.10 * reveal);
    draw_ring_soft(
        fb,
        p.x as i32,
        p.y as i32,
        24 + pulse as i32,
        GLASS,
        0.36 * reveal,
    );
    draw_disc_soft(fb, p.x as i32, p.y as i32, 10, GLASS, 0.86 * reveal);
    draw_text(
        fb,
        p.x as i32 - 46,
        p.y as i32 - 54,
        "REQUIRED",
        GLASS,
        2,
        0.90 * reveal,
    );
}

fn draw_candidate_arcs(
    fb: &mut Framebuffer,
    plan: &RoutePlan,
    route: &RouteState,
    reveal: f32,
    search_progress: Option<f32>,
) {
    let required = plan.station_position(plan.required_station);
    for candidate in &route.candidates {
        let prev = plan.station_position(route.initial_route()[candidate.position - 1]);
        let next = plan.station_position(route.initial_route()[candidate.position]);
        let selected = candidate.position == route.selected_position;
        let scored = search_progress.unwrap_or(0.0) > candidate.score;
        let color = if selected && scored {
            GOLD
        } else if scored {
            mix(PANIC, GOLD, 0.35)
        } else {
            GLASS
        };
        let alpha = if selected && scored {
            0.62
        } else if scored {
            0.23
        } else {
            0.18
        } * reveal;
        draw_curve(
            fb,
            prev,
            required,
            next,
            color,
            alpha,
            if selected { 2 } else { 1 },
        );
        if search_progress.is_some() {
            let label_x = ((prev.x + next.x + required.x) / 3.0) as i32 - 14;
            let label_y = ((prev.y + next.y + required.y) / 3.0) as i32 - 18;
            draw_text(
                fb,
                label_x,
                label_y,
                &format!("{:.0}", candidate.score * 100.0),
                color,
                1,
                alpha + 0.2,
            );
        }
    }
}

fn draw_splice(
    fb: &mut Framebuffer,
    plan: &RoutePlan,
    route: &RouteState,
    scene: SceneState,
    t: f32,
) {
    let splice = smoothstep(scene.progress);
    let (old_a, old_b) = route.selected_link();
    let a = plan.station_position(old_a);
    let b = plan.station_position(old_b);
    draw_route(fb, plan, route.initial_route(), 1.0, t, true);
    draw_line_soft(
        fb,
        (a.x as i32, a.y as i32),
        (b.x as i32, b.y as i32),
        PANIC,
        0.42 * (1.0 - splice),
        4,
    );

    let required = plan.station_position(plan.required_station);
    let start = Vec2::new(
        lerp(required.x, (a.x + b.x) * 0.5, 1.0 - splice),
        lerp(required.y, (a.y + b.y) * 0.5, 1.0 - splice),
    );
    draw_disc_soft(fb, start.x as i32, start.y as i32, 11, GLASS, 0.85);
    draw_line_soft(
        fb,
        (a.x as i32, a.y as i32),
        (start.x as i32, start.y as i32),
        GOLD,
        splice * 0.68,
        3,
    );
    draw_line_soft(
        fb,
        (start.x as i32, start.y as i32),
        (b.x as i32, b.y as i32),
        GOLD,
        splice * 0.68,
        3,
    );
    draw_text(
        fb,
        start.x as i32 - 40,
        start.y as i32 - 52,
        "SPLICE",
        MINT,
        2,
        0.70 + splice * 0.2,
    );
}

fn draw_constraint_badges(
    fb: &mut Framebuffer,
    plan: &RoutePlan,
    route: &RouteState,
    progress: f32,
    t: f32,
) {
    for badge in &route.constraint_badges {
        let local = smoothstep((progress - badge.delay) / 0.34);
        if local <= 0.0 {
            continue;
        }
        let p = plan.station_position(badge.station);
        let x = p.x as i32 + 18;
        let y = p.y as i32 - 40 + (badge.delay * 80.0) as i32;
        let color = mix(GLASS, EMERALD, local);
        let pulse = ((t * 7.0 + badge.delay * 9.0).sin() * 0.5 + 0.5) * (1.0 - local) * 0.28;
        draw_filled_rect(fb, x - 8, y - 6, 62, 22, STONE_DARK, 0.72);
        draw_rect_outline(fb, x - 8, y - 6, 62, 22, color, 0.62 + pulse);
        draw_text(fb, x, y, badge.label, color, 1, 0.86);
    }
}

fn draw_reroute(
    fb: &mut Framebuffer,
    plan: &RoutePlan,
    route: &RouteState,
    scene: SceneState,
    t: f32,
) {
    let reroute = smoothstep(scene.progress);
    draw_route(fb, plan, route.inserted_route(), 1.0, t, true);
    let (blocked_a, blocked_b) = route.blocked_link(plan);
    let a = plan.station_position(blocked_a);
    let b = plan.station_position(blocked_b);
    let mid = Vec2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    draw_line_soft(
        fb,
        (a.x as i32, a.y as i32),
        (b.x as i32, b.y as i32),
        PANIC,
        0.48,
        5,
    );
    draw_filled_rect(
        fb,
        mid.x as i32 - 32,
        mid.y as i32 - 12,
        64,
        24,
        PANIC,
        0.52,
    );
    draw_text(
        fb,
        mid.x as i32 - 20,
        mid.y as i32 - 4,
        "BLOCK",
        MINT,
        1,
        0.9,
    );
    draw_route(fb, plan, route.rerouted_route(), reroute, t, true);
    draw_text(fb, 86, 642, "ONLY THE AFFECTED LINKS REDRAW", MINT, 2, 0.82);
}

fn draw_final_lock(fb: &mut Framebuffer, plan: &RoutePlan, scene: SceneState, t: f32) {
    let goal = plan.station_position(5);
    let glow = if matches!(scene.id, SceneId::FinalRoute) {
        smoothstep(scene.progress)
    } else {
        1.0
    };
    draw_radial_glow(fb, goal.x, goal.y, 220.0, GOLD, 0.22 * glow);
    for radius in [34, 52, 72, 96] {
        draw_ring_soft(
            fb,
            goal.x as i32,
            goal.y as i32,
            radius,
            mix(GOLD, MINT, 0.25),
            0.42 * glow,
        );
    }
    for i in 0..14 {
        let angle = i as f32 / 14.0 * std::f32::consts::TAU + t * 0.08;
        draw_line_soft(
            fb,
            (goal.x as i32, goal.y as i32),
            (
                (goal.x + angle.cos() * 118.0) as i32,
                (goal.y + angle.sin() * 72.0) as i32,
            ),
            GOLD,
            0.18 * glow,
            1,
        );
    }
}

fn draw_captions(fb: &mut Framebuffer, scene: SceneState) {
    let (title, subtitle) = match scene.id {
        SceneId::Invocation => (
            "SOLVERFORGE PRESENTS",
            "ORDERED MOVEMENT THROUGH CONSTRAINED SPACE",
        ),
        SceneId::Stations => ("THE ROUTE IS A LIST", "STATIONS LIGHT IN ROUTE ORDER"),
        SceneId::Successors => (
            "EACH STATION HAS A SUCCESSOR",
            "PREV NEXT LINKS CARRY THE ROUTE",
        ),
        SceneId::RequiredStation => (
            "A REQUIRED STATION APPEARS",
            "THE LIST MUST ACCEPT A NEW VISIT",
        ),
        SceneId::InsertionSearch => ("EVALUATE INSERTION POSITIONS", "THE BEST LOCAL SPLICE WINS"),
        SceneId::Splice => ("BREAK ONE LINK FORM TWO", "DOWNSTREAM INDICES SHIFT"),
        SceneId::ConstraintRepair => ("ONLY LOCAL CONSTRAINTS PULSE", "TIME CAP ORDER DISTANCE"),
        SceneId::Reroute => ("A LINK IS BLOCKED", "A COMPACT DETOUR REPAIRS THE ROUTE"),
        SceneId::FinalRoute => ("THE ORDERED PATH IS VALID", "THE ROUTE LOCKS"),
        SceneId::Scrolltext => ("LIST VARIABLES ARE ORDERED SPACE", "SOLVERFORGE"),
    };

    draw_text(fb, 58, 36, title, MINT, 3, 0.86);
    draw_text(fb, 60, 82, subtitle, GOLD, 2, 0.76);
    let progress_w = 214;
    let x = fb.width as i32 - progress_w - 56;
    let y = fb.height as i32 - 38;
    draw_rect_outline(fb, x, y, progress_w, 12, EMERALD_DARK, 0.72);
    draw_filled_rect(
        fb,
        x + 1,
        y + 1,
        ((progress_w - 2) as f32 * scene.progress) as i32,
        10,
        EMERALD,
        0.72,
    );
}

fn draw_scrolltext(fb: &mut Framebuffer, scene: SceneState) {
    let thesis = "LIST VARIABLES ARE ORDERED SPACE";
    let hold = 5.8;
    if scene.local_t < hold {
        let alpha = smoothstep(scene.local_t / 0.7) * smoothstep((hold - scene.local_t) / 0.8);
        draw_centered_text(fb, fb.height as i32 - 112, thesis, MINT, 3, 0.92 * alpha);
        draw_centered_text(
            fb,
            fb.height as i32 - 70,
            "ONE SPLICE SETTLES THE ROUTE",
            GOLD,
            2,
            0.66 * alpha,
        );
        return;
    }

    let text = "ONE REQUIRED STATION ENTERS THE ROUTE ... ONE SUCCESSOR LINK BREAKS ... TWO LINKS FORM ... DOWNSTREAM INDICES SHIFT ... LOCAL ROUTE CONSTRAINTS REPAIR ... SOLVERFORGE BERGAMO MMXXVI ... ";
    let speed = 22.0;
    let width = text_width(text, 2);
    let mut x = 70 - ((scene.local_t - hold) * speed) as i32;
    while x < fb.width as i32 {
        draw_text(fb, x, fb.height as i32 - 92, text, MINT, 2, 0.9);
        x += width + 80;
    }
}

fn draw_centered_text(
    fb: &mut Framebuffer,
    y: i32,
    text: &str,
    color: Color,
    scale: i32,
    alpha: f32,
) {
    let x = (fb.width as i32 - text_width(text, scale)) / 2;
    draw_text(fb, x, y, text, color, scale, alpha);
}

fn role_color(role: StationRole) -> Color {
    match role {
        StationRole::Start => MINT,
        StationRole::Visit => mix(STONE, EMERALD, 0.42),
        StationRole::Required => GLASS,
        StationRole::Junction => mix(GLASS, GOLD, 0.34),
        StationRole::Checkpoint => mix(EMERALD, GOLD, 0.34),
        StationRole::Detour => EMERALD,
        StationRole::Goal => GOLD,
    }
}

fn draw_moving_arrow(fb: &mut Framebuffer, a: Vec2, b: Vec2, t: f32) {
    let p = (t * 0.35).fract();
    let x = lerp(a.x, b.x, p);
    let y = lerp(a.y, b.y, p);
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let ux = dx / len;
    let uy = dy / len;
    let lx = -uy;
    let ly = ux;
    let tip = Vec2::new(x + ux * 11.0, y + uy * 11.0);
    let left = Vec2::new(x - ux * 8.0 + lx * 6.0, y - uy * 8.0 + ly * 6.0);
    let right = Vec2::new(x - ux * 8.0 - lx * 6.0, y - uy * 8.0 - ly * 6.0);
    draw_line_soft(
        fb,
        (tip.x as i32, tip.y as i32),
        (left.x as i32, left.y as i32),
        MINT,
        0.72,
        1,
    );
    draw_line_soft(
        fb,
        (tip.x as i32, tip.y as i32),
        (right.x as i32, right.y as i32),
        MINT,
        0.72,
        1,
    );
}

fn draw_curve(
    fb: &mut Framebuffer,
    a: Vec2,
    control: Vec2,
    b: Vec2,
    color: Color,
    alpha: f32,
    thickness: i32,
) {
    let steps = 34;
    let mut prev = a;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let inv = 1.0 - t;
        let p = Vec2::new(
            inv * inv * a.x + 2.0 * inv * t * control.x + t * t * b.x,
            inv * inv * a.y + 2.0 * inv * t * control.y + t * t * b.y,
        );
        draw_line_soft(
            fb,
            (prev.x as i32, prev.y as i32),
            (p.x as i32, p.y as i32),
            color,
            alpha,
            thickness,
        );
        prev = p;
    }
}

fn draw_post(fb: &mut Framebuffer) {
    for y in (0..fb.height).step_by(2) {
        for x in 0..fb.width {
            let idx = (y * fb.width + x) * 3;
            fb.pixels[idx] = fb.pixels[idx].saturating_sub(6);
            fb.pixels[idx + 1] = fb.pixels[idx + 1].saturating_sub(6);
            fb.pixels[idx + 2] = fb.pixels[idx + 2].saturating_sub(6);
        }
    }

    for x in 0..fb.width as i32 {
        for y in 0..24 {
            fb.blend_pixel(x, y, VOID, 0.38);
            fb.blend_pixel(x, fb.height as i32 - 1 - y, VOID, 0.30);
        }
    }
}

fn draw_radial_glow(fb: &mut Framebuffer, cx: f32, cy: f32, radius: f32, color: Color, alpha: f32) {
    let min_x = (cx - radius).floor() as i32;
    let max_x = (cx + radius).ceil() as i32;
    let min_y = (cy - radius).floor() as i32;
    let max_y = (cy + radius).ceil() as i32;
    let r2 = radius * radius;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d2 = dx * dx + dy * dy;
            if d2 <= r2 {
                let falloff = 1.0 - d2.sqrt() / radius;
                fb.add_pixel(x, y, color, falloff * falloff * alpha);
            }
        }
    }
}

fn draw_ring_soft(fb: &mut Framebuffer, cx: i32, cy: i32, radius: i32, color: Color, alpha: f32) {
    let mut x = radius;
    let mut y = 0;
    let mut err = 0;
    while x >= y {
        for (dx, dy) in [
            (x, y),
            (y, x),
            (-y, x),
            (-x, y),
            (-x, -y),
            (-y, -x),
            (y, -x),
            (x, -y),
        ] {
            fb.blend_pixel(cx + dx, cy + dy, color, alpha);
            fb.blend_pixel(cx + dx + 1, cy + dy, color, alpha * 0.25);
            fb.blend_pixel(cx + dx, cy + dy + 1, color, alpha * 0.25);
        }
        y += 1;
        err += 1 + 2 * y;
        if 2 * (err - x) + 1 > 0 {
            x -= 1;
            err += 1 - 2 * x;
        }
    }
}

fn draw_disc_soft(fb: &mut Framebuffer, cx: i32, cy: i32, radius: i32, color: Color, alpha: f32) {
    for y in -radius..=radius {
        for x in -radius..=radius {
            let d2 = x * x + y * y;
            if d2 <= radius * radius {
                let falloff = 1.0 - (d2 as f32).sqrt() / radius.max(1) as f32;
                fb.blend_pixel(cx + x, cy + y, color, alpha * falloff.max(0.16));
            }
        }
    }
}

fn draw_line_soft(
    fb: &mut Framebuffer,
    start: (i32, i32),
    end: (i32, i32),
    color: Color,
    alpha: f32,
    thickness: i32,
) {
    let (mut x0, mut y0) = start;
    let (x1, y1) = end;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        for oy in -thickness..=thickness {
            for ox in -thickness..=thickness {
                let weight = if ox == 0 && oy == 0 {
                    alpha
                } else {
                    alpha * 0.24
                };
                fb.blend_pixel(x0 + ox, y0 + oy, color, weight);
            }
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn draw_filled_rect(
    fb: &mut Framebuffer,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: Color,
    alpha: f32,
) {
    if w <= 0 || h <= 0 {
        return;
    }
    for yy in y..(y + h) {
        for xx in x..(x + w) {
            fb.blend_pixel(xx, yy, color, alpha);
        }
    }
}

fn draw_rect_outline(
    fb: &mut Framebuffer,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: Color,
    alpha: f32,
) {
    draw_line_soft(fb, (x, y), (x + w, y), color, alpha, 1);
    draw_line_soft(fb, (x + w, y), (x + w, y + h), color, alpha, 1);
    draw_line_soft(fb, (x + w, y + h), (x, y + h), color, alpha, 1);
    draw_line_soft(fb, (x, y + h), (x, y), color, alpha, 1);
}

fn draw_text(
    fb: &mut Framebuffer,
    x: i32,
    y: i32,
    text: &str,
    color: Color,
    scale: i32,
    alpha: f32,
) {
    let mut cursor = x;
    for ch in text.chars() {
        draw_glyph(fb, cursor, y, ch, color, scale, alpha);
        cursor += if ch == ' ' { 4 * scale } else { 6 * scale };
    }
}

fn text_width(text: &str, scale: i32) -> i32 {
    text.chars()
        .map(|ch| if ch == ' ' { 4 * scale } else { 6 * scale })
        .sum()
}

fn draw_glyph(
    fb: &mut Framebuffer,
    x: i32,
    y: i32,
    ch: char,
    color: Color,
    scale: i32,
    alpha: f32,
) {
    for (row_idx, row) in glyph_rows(ch).iter().enumerate() {
        for col in 0..5 {
            if (row >> (4 - col)) & 1 == 0 {
                continue;
            }
            for yy in 0..scale {
                for xx in 0..scale {
                    fb.blend_pixel(
                        x + col * scale + xx,
                        y + row_idx as i32 * scale + yy,
                        color,
                        alpha,
                    );
                }
            }
        }
    }
}

fn glyph_rows(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0E],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' => [0x01, 0x01, 0x01, 0x01, 0x11, 0x11, 0x0E],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '3' => [0x1E, 0x01, 0x01, 0x0E, 0x01, 0x01, 0x1E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x10, 0x1E, 0x01, 0x01, 0x1E],
        '6' => [0x0E, 0x10, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x01, 0x0E],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C, 0x08],
        ':' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00],
        '/' => [0x01, 0x02, 0x04, 0x04, 0x08, 0x10, 0x00],
        '\'' => [0x0C, 0x0C, 0x08, 0x00, 0x00, 0x00, 0x00],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04],
        '>' => [0x10, 0x08, 0x04, 0x02, 0x04, 0x08, 0x10],
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        _ => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    }
}
