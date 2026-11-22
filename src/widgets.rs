//! Paleta, fontes e widgets reutilizáveis (campos, botões, chips, ícones).

use crate::util::*;
use eframe::egui::{
    self, emath::easing, pos2, vec2, Align2, Color32, FontData, FontDefinitions, FontFamily,
    FontId, Id, Mesh, Painter, Pos2, Rect, Response, RichText, Sense, Shape, TextureHandle, Ui,
};
use std::f32::consts::PI;

// ───────────────────────────── Paleta ─────────────────────────────

pub const BG: Color32 = Color32::from_rgb(4, 6, 9);
pub const PANEL: Color32 = Color32::from_rgb(10, 14, 19);
pub const PANEL2: Color32 = Color32::from_rgb(16, 22, 30);
pub const LINE: Color32 = Color32::from_rgb(30, 40, 52);
pub const TEXT: Color32 = Color32::from_rgb(232, 238, 244);
pub const MUTED: Color32 = Color32::from_rgb(130, 144, 158);
pub const DIM: Color32 = Color32::from_rgb(82, 95, 108);
pub const ACCENT: Color32 = Color32::from_rgb(34, 211, 238);
pub const GOLD: Color32 = Color32::from_rgb(250, 204, 21);
pub const DANGER: Color32 = Color32::from_rgb(251, 113, 133);
pub const OK: Color32 = Color32::from_rgb(74, 222, 128);
pub const VIOLET: Color32 = Color32::from_rgb(139, 108, 255);

// ───────────────────────────── Fontes ─────────────────────────────

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}

pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

pub fn sans(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "inter".into(),
        FontData::from_static(include_bytes!("../assets/fonts/Inter.ttf")),
    );
    fonts.font_data.insert(
        "jb_mono".into(),
        FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf")),
    );
    fonts.font_data.insert(
        "jb_bold".into(),
        FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf")),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "inter".into());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "jb_mono".into());
    let mut bold_family = vec!["jb_bold".to_string()];
    bold_family.extend(
        fonts
            .families
            .get(&FontFamily::Monospace)
            .cloned()
            .unwrap_or_default(),
    );
    fonts
        .families
        .insert(FontFamily::Name("bold".into()), bold_family);
    ctx.set_fonts(fonts);
}

pub fn setup_style(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = BG;
    v.extreme_bg_color = PANEL;
    v.selection.bg_fill = alpha(ACCENT, 0.35);
    v.selection.stroke = st(1.0_f32, TEXT);
    v.widgets.noninteractive.fg_stroke = st(1.0_f32, Color32::from_gray(215));
    v.widgets.inactive.fg_stroke = st(1.0_f32, Color32::from_gray(220));
    v.widgets.inactive.bg_stroke = st(1.0_f32, LINE);
    v.widgets.hovered.bg_stroke = st(1.0_f32, MUTED);
    v.widgets.active.bg_stroke = st(1.0_f32, ACCENT);
    v.text_cursor.stroke = st(2.0_f32, ACCENT);
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.scroll = egui::style::ScrollStyle {
            bar_width: 6.0,
            floating: true,
            floating_width: 4.0,
            floating_allocated_width: 0.0,
            ..egui::style::ScrollStyle::floating()
        };
    });
}

// ───────────────────────────── Desenho ─────────────────────────────

/// Gradiente linear de 4 cantos.
pub fn gradient(p: &Painter, r: Rect, tl: Color32, tr: Color32, br: Color32, bl: Color32) {
    let mut m = Mesh::default();
    m.colored_vertex(r.left_top(), tl);
    m.colored_vertex(r.right_top(), tr);
    m.colored_vertex(r.right_bottom(), br);
    m.colored_vertex(r.left_bottom(), bl);
    m.add_triangle(0, 1, 2);
    m.add_triangle(0, 2, 3);
    p.add(Shape::mesh(m));
}

pub fn hgrad(p: &Painter, r: Rect, left: Color32, right: Color32) {
    gradient(p, r, left, right, right, left);
}

/// Brilho suave (círculos concêntricos somados à imagem de fundo).
pub fn glow(p: &Painter, c: Pos2, radius: f32, color: Color32, strength: f32) {
    let n = 22;
    for i in 0..n {
        let k = i as f32 / n as f32;
        let rad = radius * (1.0 - k);
        let a = strength * (1.0 / n as f32) * (0.5 + k) * 1.3;
        let add = Color32::from_rgb(
            (color.r() as f32 * a).round() as u8,
            (color.g() as f32 * a).round() as u8,
            (color.b() as f32 * a).round() as u8,
        )
        .additive();
        p.circle_filled(c, rad, add);
    }
}

pub fn star_icon(p: &Painter, c: Pos2, r: f32, filled: bool, color: Color32) {
    let mut pts = Vec::with_capacity(10);
    for i in 0..10 {
        let a = -PI / 2.0 + i as f32 * PI / 5.0;
        let rr = if i % 2 == 0 { r } else { r * 0.42 };
        pts.push(pos2(c.x + rr * a.cos(), c.y + rr * a.sin()));
    }
    if filled {
        let mut m = Mesh::default();
        m.colored_vertex(c, color);
        for q in &pts {
            m.colored_vertex(*q, color);
        }
        for i in 0..10u32 {
            m.add_triangle(0, i + 1, (i + 1) % 10 + 1);
        }
        p.add(Shape::mesh(m));
    }
    p.add(Shape::closed_line(pts, st(1.4, color)));
}

pub fn eye_icon(p: &Painter, c: Pos2, open: bool, color: Color32) {
    let w = 9.0;
    let h = 5.0;
    let n = 12;
    let mut top = Vec::new();
    let mut bot = Vec::new();
    for i in 0..=n {
        let u = i as f32 / n as f32;
        let x = c.x - w + 2.0 * w * u;
        let y = h * (PI * u).sin();
        top.push(pos2(x, c.y - y));
        bot.push(pos2(x, c.y + y));
    }
    p.add(Shape::line(top, st(1.4, color)));
    p.add(Shape::line(bot, st(1.4, color)));
    p.circle_filled(c, 2.2, color);
    if !open {
        p.line_segment([c + vec2(-8.0, 8.0), c + vec2(8.0, -8.0)], st(1.6, color));
    }
}

pub fn search_icon(p: &Painter, c: Pos2, color: Color32) {
    p.circle_stroke(c + vec2(-1.5, -1.5), 5.5, st(1.6, color));
    p.line_segment([c + vec2(2.5, 2.5), c + vec2(7.0, 7.0)], st(1.8, color));
}

pub fn close_icon(p: &Painter, c: Pos2, d: f32, color: Color32) {
    p.line_segment([c + vec2(-d, -d), c + vec2(d, d)], st(1.8, color));
    p.line_segment([c + vec2(-d, d), c + vec2(d, -d)], st(1.8, color));
}

pub fn arrow_icon(p: &Painter, c: Pos2, dir: f32, color: Color32) {
    let s = 5.0;
    p.line_segment([c + vec2(-dir * s, -s * 1.4), c + vec2(dir * s, 0.0)], st(1.8, color));
    p.line_segment([c + vec2(dir * s, 0.0), c + vec2(-dir * s, s * 1.4)], st(1.8, color));
}

/// Imagem dentro de um círculo (avatar) ou letra inicial.
pub fn draw_avatar(
    p: &Painter,
    c: Pos2,
    r: f32,
    tex: Option<&TextureHandle>,
    letter: &str,
    fade: f32,
) {
    match tex {
        Some(t) => {
            let mut m = Mesh::with_texture(t.id());
            let col = gray(255, fade);
            let n = 48u32;
            m.vertices.push(egui::epaint::Vertex {
                pos: c,
                uv: pos2(0.5, 0.5),
                color: col,
            });
            for i in 0..=n {
                let a = i as f32 / n as f32 * 2.0 * PI;
                m.vertices.push(egui::epaint::Vertex {
                    pos: pos2(c.x + r * a.cos(), c.y + r * a.sin()),
                    uv: pos2(0.5 + 0.5 * a.cos(), 0.5 + 0.5 * a.sin()),
                    color: col,
                });
            }
            for i in 0..n {
                m.indices.extend_from_slice(&[0, i + 1, i + 2]);
            }
            p.add(Shape::mesh(m));
        }
        None => {
            p.circle_filled(c, r, fc(PANEL2, fade));
            p.text(c, Align2::CENTER_CENTER, letter, bold(r * 0.95), fc(ACCENT, fade));
        }
    }
    p.circle_stroke(c, r, st(1.5, fc(ACCENT, fade)));
}

// ───────────────────────────── Widgets ─────────────────────────────

fn hand(ui: &Ui, resp: &Response) {
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
}

/// Campo de texto com borda animada ao focar. `left`/`right` reservam
/// espaço para ícones desenhados por quem chama.
#[allow(clippy::too_many_arguments)]
pub fn field(
    ui: &mut Ui,
    id: Id,
    text: &mut String,
    hint: &str,
    password: bool,
    w: f32,
    h: f32,
    fade: f32,
    left: f32,
    right: f32,
) -> (Rect, Response) {
    let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
    let focused = ui.ctx().memory(|m| m.has_focus(id));
    let f = ui
        .ctx()
        .animate_bool_with_time(id.with("focus"), focused, 0.18);
    let p = ui.painter().clone();
    if f > 0.01 {
        p.rect_stroke(
            rect.expand(3.0),
            10.0,
            st(3.0, alpha(ACCENT, 0.16 * f * fade)),
        );
    }
    p.rect_filled(rect, 8.0, mix(fc(PANEL, fade), fc(PANEL2, fade), f));
    p.rect_stroke(
        rect,
        8.0,
        st(1.0 + 0.5 * f, mix(fc(LINE, fade), fc(ACCENT, fade), f)),
    );
    let inner = Rect::from_min_max(
        rect.min + vec2(left, 2.0),
        rect.max - vec2(right, 2.0),
    );
    let resp = ui.put(
        inner,
        egui::TextEdit::singleline(text)
            .id(id)
            .password(password)
            .hint_text(RichText::new(hint).color(fc(DIM, fade)))
            .font(mono(14.5))
            .text_color(fc(TEXT, fade))
            .frame(false)
            .margin(vec2(12.0, (h - 4.0 - 18.0) / 2.0))
            .desired_width(inner.width()),
    );
    (rect, resp)
}

/// Chip de filtro. Retorna true se foi clicado.
pub fn chip(
    ui: &mut Ui,
    label: &str,
    count: Option<usize>,
    selected: bool,
    color: Color32,
    dot: bool,
) -> bool {
    let font = mono(12.5);
    let cfont = mono(11.0);
    let lw = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), Color32::WHITE)
        .size()
        .x;
    let cw = count
        .map(|n| {
            ui.painter()
                .layout_no_wrap(n.to_string(), cfont.clone(), Color32::WHITE)
                .size()
                .x
                + 8.0
        })
        .unwrap_or(0.0);
    let dw = if dot { 14.0 } else { 0.0 };
    let size = vec2(lw + cw + dw + 28.0, 32.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let sel = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("sel"), selected, 0.22);
    let hov = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("hov"), resp.hovered(), 0.12);
    hand(ui, &resp);
    let p = ui.painter();
    let fill = mix(mix(PANEL, PANEL2, hov), mix(PANEL2, color, 0.2), sel);
    p.rect_filled(rect, 16.0, fill);
    p.rect_stroke(
        rect,
        16.0,
        st(1.0, mix(mix(LINE, MUTED, hov), color, sel)),
    );
    let mut x = rect.left() + 14.0;
    if dot {
        p.circle_filled(pos2(x + 2.0, rect.center().y), 3.0 + 0.8 * sel, color);
        x += 14.0;
    }
    let txt = mix(mix(MUTED, TEXT, hov), TEXT, sel);
    p.text(pos2(x, rect.center().y), Align2::LEFT_CENTER, label, font, txt);
    if let Some(n) = count {
        p.text(
            pos2(x + lw + 8.0, rect.center().y + 0.5),
            Align2::LEFT_CENTER,
            n.to_string(),
            cfont,
            mix(DIM, color, sel),
        );
    }
    resp.clicked()
}

/// Botão principal (preenchimento acompanha o mouse).
pub fn big_button(ui: &mut Ui, label: &str, w: f32, fade: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 46.0), Sense::click());
    let hov = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.2);
    let down = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("d"), resp.is_pointer_button_down_on(), 0.08);
    hand(ui, &resp);
    let p = ui.painter();
    let r = rect.shrink(1.5 * down);
    if hov > 0.01 {
        p.rect_stroke(r.expand(3.0), 11.0, st(3.0, alpha(ACCENT, 0.18 * hov * fade)));
    }
    p.rect_filled(r, 9.0, fc(mix(PANEL2, ACCENT, 0.08 + 0.1 * hov), fade));
    p.rect_stroke(r, 9.0, st(1.5, fc(ACCENT, fade)));
    let fw = r.width() * easing::cubic_out(hov);
    let filled = Rect::from_min_size(r.min, vec2(fw, r.height()));
    p.rect_filled(filled, 9.0, fc(ACCENT, fade));
    let font = bold(14.5);
    p.text(r.center(), Align2::CENTER_CENTER, label, font.clone(), fc(TEXT, fade));
    p.with_clip_rect(filled)
        .text(r.center(), Align2::CENTER_CENTER, label, font, Color32::from_rgb(2, 20, 26));
    resp.clicked()
}

/// Botão secundário pequeno (contorno). `star` desenha uma estrela
/// (cheia ou vazada) à esquerda do texto. Retorna true se clicado.
pub fn ghost_button(
    ui: &mut Ui,
    label: &str,
    w: f32,
    active: bool,
    color: Color32,
    star: Option<bool>,
    fade: f32,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 36.0), Sense::click());
    let hov = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.15);
    let act = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("a"), active, 0.2);
    hand(ui, &resp);
    let p = ui.painter();
    p.rect_filled(
        rect,
        8.0,
        fc(mix(mix(PANEL, PANEL2, hov), mix(PANEL2, color, 0.18), act), fade),
    );
    p.rect_stroke(
        rect,
        8.0,
        st(1.0, fc(mix(mix(LINE, MUTED, hov), color, act), fade)),
    );
    let txt = fc(mix(mix(MUTED, TEXT, hov), TEXT, act), fade);
    let mut cx = rect.center().x;
    if let Some(filled) = star {
        let lw = p
            .layout_no_wrap(label.to_string(), mono(12.0), Color32::WHITE)
            .size()
            .x;
        let total = lw + 22.0;
        let x0 = rect.center().x - total / 2.0;
        star_icon(p, pos2(x0 + 7.0, rect.center().y), 7.0 + 1.0 * hov, filled, fc(color, fade));
        cx = x0 + 22.0 + lw / 2.0;
    }
    p.text(pos2(cx, rect.center().y), Align2::CENTER_CENTER, label, mono(12.0), txt);
    resp.clicked()
}

pub fn menu_item(ui: &mut Ui, label: &str, w: f32, danger: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 38.0), Sense::click());
    let hov = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.15);
    hand(ui, &resp);
    let base = if danger { DANGER } else { ACCENT };
    let p = ui.painter();
    p.rect_filled(rect, 6.0, alpha(base, 0.16 * hov));
    if hov > 0.01 {
        p.rect_filled(
            Rect::from_min_size(rect.min + vec2(0.0, 8.0), vec2(2.5, rect.height() - 16.0)),
            1.0,
            fc(base, hov),
        );
    }
    p.text(
        rect.left_center() + vec2(14.0 + 4.0 * hov, 0.0),
        Align2::LEFT_CENTER,
        label,
        mono(12.5),
        mix(Color32::from_gray(200), if danger { DANGER } else { TEXT }, hov),
    );
    resp.clicked()
}

/// Número pseudo-aleatório estável entre 0 e 1 (mesma entrada, mesmo valor).
pub fn hash01(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x9E37_79B1);
    x ^= x >> 15;
    x = x.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 13;
    (x & 0xFFFF) as f32 / 65535.0
}

/// Fundo animado: grade suave, varredura, partículas flutuantes, brilhos nos
/// cantos e uma luz suave que acompanha o mouse.
pub fn draw_background(ctx: &egui::Context, time: f64) {
    let r = ctx.screen_rect();
    let p = ctx.layer_painter(egui::LayerId::background());
    p.rect_filled(r, 0.0, BG);
    glow(
        &p,
        pos2(r.left() + r.width() * 0.12, r.top() - 40.0),
        r.width().max(r.height()) * 0.55,
        ACCENT,
        0.10,
    );
    glow(
        &p,
        pos2(r.right(), r.bottom()),
        r.width().max(r.height()) * 0.45,
        Color32::from_rgb(120, 90, 255),
        0.07,
    );
    let gap = 44.0_f32;
    let off = ((time * 5.0) % gap as f64) as f32;
    let line = st(1.0, Color32::from_rgba_unmultiplied(120, 170, 200, 9));
    let nx = (r.width() / gap) as i32 + 2;
    let ny = (r.height() / gap) as i32 + 2;
    for i in 0..nx {
        let x = r.left() + gap * i as f32 + off - gap;
        p.line_segment([pos2(x, r.top()), pos2(x, r.bottom())], line);
    }
    for j in 0..ny {
        let y = r.top() + gap * j as f32 + off - gap;
        p.line_segment([pos2(r.left(), y), pos2(r.right(), y)], line);
    }

    // luz que acompanha o mouse, com atraso suave
    if let Some(m) = ctx.input(|i| i.pointer.hover_pos()) {
        let x = ctx.animate_value_with_time(Id::new("bg_mx"), m.x, 0.6);
        let y = ctx.animate_value_with_time(Id::new("bg_my"), m.y, 0.6);
        glow(&p, pos2(x, y), 320.0, ACCENT, 0.05);
    }

    // partículas que sobem devagar e piscam
    let (w, h) = (r.width().max(1.0), r.height().max(1.0));
    for i in 0..46u32 {
        let fx = hash01(i * 3 + 1);
        let fy = hash01(i * 3 + 2);
        let speed = 5.0 + 14.0 * hash01(i * 3 + 3);
        let size = 0.7 + 1.5 * hash01(i * 7 + 5);
        let t = time as f32;
        let rise = (fy * h + t * speed) % (h + 24.0);
        let y = r.bottom() + 12.0 - rise;
        let x = r.left() + fx * w + (t * 0.35 + i as f32).sin() * 14.0;
        let tw = 0.5 + 0.5 * (t * (0.8 + hash01(i) * 1.4) + i as f32 * 1.7).sin();
        // some perto das bordas verticais (entrada e saída suaves)
        let edge = ((y - r.top()) / 90.0).clamp(0.0, 1.0) * ((r.bottom() - y) / 90.0).clamp(0.0, 1.0);
        let col = if i % 3 == 0 { VIOLET } else { ACCENT };
        p.circle_filled(pos2(x, y), size, alpha(col, (0.10 + 0.32 * tw) * edge));
    }

    let sy = r.top() + ((time * 80.0) % (r.height() as f64 + 200.0)) as f32 - 100.0;
    for k in 0..12 {
        let a = 0.07 * (1.0 - k as f32 / 12.0);
        let y = sy - k as f32 * 3.0;
        p.line_segment(
            [pos2(r.left(), y), pos2(r.right(), y)],
            st(1.0, alpha(ACCENT, a)),
        );
    }
}

/// Trilhas de circuito (estilo PCB) com pulsos de luz que viajam por elas.
/// Ficam nas bordas de cima e de baixo, como decoração discreta.
pub fn circuit_traces(p: &Painter, area: Rect, time: f64, fade: f32) {
    const TRACES: [&[(f32, f32)]; 6] = [
        &[(0.00, 0.10), (0.10, 0.10), (0.14, 0.14), (0.30, 0.14), (0.34, 0.10), (0.46, 0.10)],
        &[(0.00, 0.17), (0.06, 0.17), (0.10, 0.21), (0.22, 0.21)],
        &[(0.00, 0.90), (0.12, 0.90), (0.16, 0.86), (0.34, 0.86), (0.38, 0.90), (0.50, 0.90)],
        &[(0.00, 0.83), (0.08, 0.83), (0.12, 0.79), (0.20, 0.79)],
        &[(1.00, 0.12), (0.90, 0.12), (0.86, 0.16), (0.70, 0.16), (0.66, 0.12), (0.56, 0.12)],
        &[(1.00, 0.88), (0.88, 0.88), (0.84, 0.84), (0.68, 0.84), (0.64, 0.88), (0.54, 0.88)],
    ];
    for (i, tr) in TRACES.iter().enumerate() {
        let pts: Vec<Pos2> = tr
            .iter()
            .map(|(x, y)| pos2(area.left() + area.width() * x, area.top() + area.height() * y))
            .collect();
        p.add(Shape::line(pts.clone(), st(1.5, alpha(ACCENT, 0.13 * fade))));
        // "pad" no fim da trilha
        let end = *pts.last().unwrap();
        p.circle_stroke(end, 4.5, st(1.4, alpha(ACCENT, 0.35 * fade)));
        p.circle_filled(end, 1.6, alpha(ACCENT, 0.5 * fade));
        // pulso com rastro
        let t = ((time as f32 * 0.11) + i as f32 * 0.29).fract();
        for j in 0..9 {
            let tt = t - j as f32 * 0.012;
            if tt < 0.0 {
                continue;
            }
            let q = along(&pts, tt);
            let a = (1.0 - j as f32 / 9.0) * fade;
            p.circle_filled(q, 2.6 - j as f32 * 0.2, alpha(ACCENT, 0.85 * a));
            if j == 0 {
                glow(p, q, 14.0, ACCENT, 0.5 * fade);
            }
        }
    }
}

/// Cadeado simples (para o painel de recursos).
pub fn lock_icon(p: &Painter, c: Pos2, color: Color32) {
    let body = Rect::from_center_size(c + vec2(0.0, 3.0), vec2(14.0, 10.0));
    p.rect_stroke(body, 2.5, st(1.7, color));
    arc(p, c + vec2(0.0, -2.0), 4.6, PI, 2.0 * PI, st(1.7, color));
    p.line_segment([c + vec2(-4.6, -2.0), c + vec2(-4.6, -0.5)], st(1.7, color));
    p.line_segment([c + vec2(4.6, -2.0), c + vec2(4.6, -0.5)], st(1.7, color));
    p.circle_filled(c + vec2(0.0, 3.0), 1.4, color);
}

/// Corta o texto com reticências para caber em `max_w` pixels.
pub fn ellipsize(p: &Painter, text: &str, font: FontId, max_w: f32) -> String {
    let width = |s: &str| {
        p.layout_no_wrap(s.to_string(), font.clone(), Color32::WHITE)
            .size()
            .x
    };
    if width(text) <= max_w {
        return text.to_string();
    }
    let mut s = text.to_string();
    while !s.is_empty() && width(&format!("{s}…")) > max_w {
        s.pop();
    }
    format!("{}…", s.trim_end())
}

/// Cantos estilo "mira" (HUD) ao redor de um retângulo.
pub fn brackets(p: &Painter, r: Rect, len: f32, stroke: egui::Stroke) {
    for (c, dx, dy) in [
        (r.left_top(), 1.0, 1.0),
        (r.right_top(), -1.0, 1.0),
        (r.left_bottom(), 1.0, -1.0),
        (r.right_bottom(), -1.0, -1.0),
    ] {
        p.line_segment([c, c + vec2(dx * len, 0.0)], stroke);
        p.line_segment([c, c + vec2(0.0, dy * len)], stroke);
    }
}

/// Faixa de brilho diagonal que atravessa `r` (k de 0 a 1), recortada no retângulo.
pub fn sweep(p: &Painter, r: Rect, k: f32, strength: f32) {
    if strength <= 0.0 {
        return;
    }
    let pc = p.with_clip_rect(r);
    let bw = r.width() * 0.35;
    let x = r.left() - bw + (r.width() + bw * 2.0) * k;
    let c = Color32::from_white_alpha((70.0 * strength) as u8);
    let band_l = Rect::from_min_max(pos2(x, r.top()), pos2(x + bw * 0.5, r.bottom()));
    let band_r = Rect::from_min_max(pos2(x + bw * 0.5, r.top()), pos2(x + bw, r.bottom()));
    hgrad(&pc, band_l, Color32::TRANSPARENT, c);
    hgrad(&pc, band_r, c, Color32::TRANSPARENT);
}

/// Medidor de força de senha (0 a 4).
pub fn password_score(pw: &str) -> usize {
    if pw.is_empty() {
        return 0;
    }
    let mut s = 0;
    if pw.chars().count() >= 6 {
        s += 1;
    }
    if pw.chars().count() >= 10 {
        s += 1;
    }
    if pw.chars().any(|c| c.is_ascii_digit()) && pw.chars().any(|c| c.is_alphabetic()) {
        s += 1;
    }
    if pw.chars().any(|c| !c.is_alphanumeric()) || pw.chars().any(|c| c.is_uppercase()) {
        s += 1;
    }
    s
}
