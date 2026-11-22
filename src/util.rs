//! Funções utilitárias de cor, geometria e desenho usadas pelo app inteiro.

use eframe::egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Stroke};

/// Escurece uma cor (fade-in sobre o fundo preto). `f` de 0 a 1.
pub fn fc(c: Color32, f: f32) -> Color32 {
    let f = f.clamp(0.0, 1.0);
    Color32::from_rgb(
        (c.r() as f32 * f) as u8,
        (c.g() as f32 * f) as u8,
        (c.b() as f32 * f) as u8,
    )
}

/// Cor com transparência. O egui mistura no espaço "gamma", então a
/// pré-multiplicação também é feita ali (previsível: 0.5 = metade).
pub fn alpha(c: Color32, a: f32) -> Color32 {
    let a = a.clamp(0.0, 1.0);
    Color32::from_rgba_premultiplied(
        (c.r() as f32 * a).round() as u8,
        (c.g() as f32 * a).round() as u8,
        (c.b() as f32 * a).round() as u8,
        (a * 255.0).round() as u8,
    )
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    a.lerp_to_gamma(b, t.clamp(0.0, 1.0))
}

/// Tom de cinza multiplicado pelo "fade" (0..1). Como tudo é preto e branco,
/// multiplicar o cinza equivale a um fade-in sobre o fundo preto.
pub fn gray(v: u8, fade: f32) -> Color32 {
    Color32::from_gray((v as f32 * fade.clamp(0.0, 1.0)) as u8)
}

pub fn st(w: f32, c: Color32) -> Stroke {
    Stroke::new(w, c)
}

pub fn fit(rect: Rect, aspect: f32, pad: f32) -> Rect {
    let r = rect.shrink(pad);
    let (mut w, mut h) = (r.width(), r.height());
    if w / h > aspect {
        w = h * aspect;
    } else {
        h = w / aspect;
    }
    Rect::from_center_size(r.center(), vec2(w, h))
}

pub fn arc(p: &Painter, c: Pos2, rad: f32, a0: f32, a1: f32, stroke: Stroke) {
    let n = 14;
    let mut last = pos2(c.x + rad * a0.cos(), c.y + rad * a0.sin());
    for i in 1..=n {
        let a = a0 + (a1 - a0) * i as f32 / n as f32;
        let q = pos2(c.x + rad * a.cos(), c.y + rad * a.sin());
        p.line_segment([last, q], stroke);
        last = q;
    }
}

/// Arco com número de segmentos proporcional ao tamanho (bom para círculos inteiros).
pub fn arc_smooth(p: &Painter, c: Pos2, rad: f32, a0: f32, a1: f32, stroke: Stroke) {
    let n = (((a1 - a0).abs() * rad) / 3.0).ceil().max(2.0) as usize;
    let mut last = pos2(c.x + rad * a0.cos(), c.y + rad * a0.sin());
    for i in 1..=n {
        let a = a0 + (a1 - a0) * i as f32 / n as f32;
        let q = pos2(c.x + rad * a.cos(), c.y + rad * a.sin());
        p.line_segment([last, q], stroke);
        last = q;
    }
}

pub fn along(pts: &[Pos2], t: f32) -> Pos2 {
    let total: f32 = pts.windows(2).map(|w| (w[1] - w[0]).length()).sum();
    let mut d = t.fract() * total;
    for w in pts.windows(2) {
        let l = (w[1] - w[0]).length();
        if d <= l {
            return w[0] + (w[1] - w[0]) * (d / l.max(0.001));
        }
        d -= l;
    }
    *pts.last().unwrap()
}
