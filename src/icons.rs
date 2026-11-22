//! Desenhos animados em código, usados quando um item não tem foto.

use crate::data::Icon;
use crate::util::*;
use eframe::egui::{pos2, vec2, Align2, FontId, Painter, Pos2, Rect};

pub fn draw_icon(p: &Painter, rect: Rect, icon: Icon, hov: f32, time: f64, fade: f32) {
    let c = |v: u8| gray(v, fade);
    let t = time as f32;
    match icon {
        Icon::Board {
            aspect,
            pins,
            usb,
            ant,
            chips,
        } => {
            let r = fit(rect, aspect, 18.0);
            p.rect_filled(r, 5.0, c(12));
            p.rect_stroke(r, 5.0, st(1.6, c(235)));
            let n = (pins as usize).max(2);
            let step = (r.width() - 34.0) / (n as f32 - 1.0);
            let active = if hov > 0.01 {
                ((t * 14.0) as usize) % n
            } else {
                usize::MAX
            };
            for i in 0..n {
                let x = r.left() + 17.0 + step * i as f32;
                let (col, rad) = if i == active {
                    (c(255), 3.4)
                } else {
                    (c(120), 2.1)
                };
                p.circle_filled(pos2(x, r.top() + 8.0), rad, col);
                p.circle_filled(pos2(x, r.bottom() - 8.0), rad, col);
            }
            if usb {
                let u = Rect::from_center_size(pos2(r.left() + 2.0, r.center().y), vec2(16.0, 20.0));
                p.rect_filled(u, 2.0, c(12));
                p.rect_stroke(u, 2.0, st(1.5, c(220)));
                p.rect_stroke(u.shrink2(vec2(4.0, 5.0)), 1.0, st(1.0, c(140)));
            }
            let ch = Rect::from_center_size(
                pos2(r.center().x + r.width() * 0.06, r.center().y),
                vec2(r.width() * 0.24, r.height() * 0.36),
            );
            p.rect_stroke(ch, 2.0, st(1.4, c(230)));
            p.circle_filled(pos2(ch.left() + 5.0, ch.top() + 5.0), 1.6, c(230));
            if chips >= 2 {
                let c2 = Rect::from_center_size(
                    pos2(ch.left() - r.width() * 0.17, r.center().y),
                    vec2(r.width() * 0.12, r.height() * 0.24),
                );
                p.rect_stroke(c2, 1.5, st(1.2, c(170)));
            }
            if ant {
                let x0 = r.right() - r.width() * 0.2;
                let seg = r.width() * 0.16 / 6.0;
                let h = r.height() * 0.16;
                let mut last = pos2(x0, r.center().y);
                for k in 1..=6 {
                    let nx = x0 + seg * k as f32;
                    let ny = r.center().y + if k % 2 == 0 { -h } else { h };
                    p.line_segment([last, pos2(nx, ny)], st(1.5, c(200)));
                    last = pos2(nx, ny);
                }
            }
            let glow = 0.5 + 0.5 * (t * (2.0 + 8.0 * hov)).sin();
            let led = pos2(r.left() + r.width() * 0.25, r.center().y + r.height() * 0.2);
            p.circle_stroke(led, 4.0, st(1.0, c(160)));
            p.circle_filled(led, 2.6, c((60.0 + 195.0 * glow) as u8));
        }
        Icon::Oled => {
            let r = fit(rect, 1.25, 18.0);
            p.rect_filled(r, 4.0, c(12));
            p.rect_stroke(r, 4.0, st(1.6, c(235)));
            for i in 0..4 {
                let x = r.left() + r.width() * (0.3 + 0.13 * i as f32);
                p.circle_filled(pos2(x, r.top() + 7.0), 2.0, c(140));
            }
            let s = Rect::from_min_max(
                pos2(r.left() + 10.0, r.top() + 16.0),
                pos2(r.right() - 10.0, r.bottom() - 9.0),
            );
            p.rect_filled(s, 2.0, c(3));
            p.rect_stroke(s, 2.0, st(1.0, c(150)));
            p.rect_filled(
                Rect::from_min_size(s.min + vec2(6.0, 5.0), vec2(s.width() * 0.4, 3.0)),
                0.0,
                c(230),
            );
            p.rect_filled(
                Rect::from_min_size(s.min + vec2(6.0, 11.0), vec2(s.width() * 0.25, 3.0)),
                0.0,
                c(120),
            );
            let speed = 1.2 + 5.0 * hov;
            let steps = 28;
            let mut last: Option<Pos2> = None;
            for i in 0..=steps {
                let u = i as f32 / steps as f32;
                let y = s.bottom()
                    - s.height() * 0.28
                    + (u * 9.0 + t * speed).sin()
                        * s.height()
                        * 0.16
                        * (0.5 + 0.5 * (u * 3.0 + t).sin());
                let q = pos2(s.left() + 6.0 + (s.width() - 12.0) * u, y);
                if let Some(l) = last {
                    p.line_segment([l, q], st(1.4, c(240)));
                }
                last = Some(q);
            }
            if hov > 0.01 {
                let x = s.left() + s.width() * (t * 0.7).fract();
                p.line_segment([pos2(x, s.top()), pos2(x, s.bottom())], st(1.0, c(90)));
            }
        }
        Icon::Lcd { rows } => {
            let rows = rows as usize;
            let cols: usize = if rows >= 4 { 20 } else { 16 };
            let aspect = if rows >= 4 { 1.75 } else { 2.3 };
            let r = fit(rect, aspect, 14.0);
            p.rect_filled(r, 4.0, c(12));
            p.rect_stroke(r, 4.0, st(1.6, c(235)));
            let s = r.shrink2(vec2(10.0, 10.0));
            p.rect_filled(s, 2.0, c(4));
            p.rect_stroke(s, 2.0, st(1.0, c(150)));
            let g = s.shrink(5.0);
            let (cw, chh) = (g.width() / cols as f32, g.height() / rows as f32);
            let total = cols * rows;
            let k = ((t * (3.0 + 14.0 * hov)) as usize) % (total + 8);
            let cell = |i: usize| {
                let (ri, ci) = (i / cols, i % cols);
                let center = pos2(
                    g.left() + cw * (ci as f32 + 0.5),
                    g.top() + chh * (ri as f32 + 0.5),
                );
                let w = cw - 2.0;
                Rect::from_center_size(center, vec2(w, (w * 1.6).min(chh - 2.0)))
            };
            for i in 0..total {
                let cl = cell(i);
                p.rect_stroke(cl, 0.0, st(0.6, c(38)));
                if i < k && (i * 7 + 3) % 5 != 0 {
                    p.rect_filled(cl.shrink(1.0), 0.0, c(215));
                }
            }
            if k < total && (t * 2.0) as i64 % 2 == 0 {
                p.rect_filled(cell(k), 0.0, c(255));
            }
        }
        Icon::Nfc => {
            let r = fit(rect, 1.25, 30.0);
            p.rect_filled(r, 4.0, c(12));
            p.rect_stroke(r, 4.0, st(1.6, c(235)));
            let ctr = r.center();
            for k in 0..4 {
                let rr = Rect::from_center_size(
                    ctr,
                    vec2(
                        r.width() * (0.78 - 0.14 * k as f32),
                        r.height() * (0.78 - 0.16 * k as f32),
                    ),
                );
                p.rect_stroke(rr, 3.0, st(1.2, c(190)));
            }
            p.rect_filled(Rect::from_center_size(ctr, vec2(10.0, 10.0)), 1.0, c(235));
            let amp = 0.35 + 0.65 * hov;
            for k in 0..3 {
                let ph = (t * 0.7 + k as f32 / 3.0) % 1.0;
                let rr = r.expand(ph * 24.0);
                p.rect_stroke(rr, 8.0, st(1.2, c((170.0 * (1.0 - ph) * amp) as u8)));
            }
        }
        Icon::Cyd => {
            let r = fit(rect, 1.45, 14.0);
            p.rect_filled(r, 5.0, c(12));
            p.rect_stroke(r, 5.0, st(1.6, c(235)));
            let s = r.shrink2(vec2(10.0, 9.0));
            p.rect_filled(s, 3.0, c(3));
            p.rect_stroke(s, 3.0, st(1.2, c(190)));
            p.rect_filled(
                Rect::from_min_size(s.min + vec2(1.0, 1.0), vec2(s.width() - 2.0, 7.0)),
                0.0,
                c(70),
            );
            let bw = (s.width() - 16.0) / 3.0;
            for i in 0..3 {
                let b = Rect::from_min_size(
                    pos2(s.left() + 8.0 + bw * i as f32, s.top() + 16.0),
                    vec2(bw - 6.0, s.height() * 0.28),
                );
                p.rect_stroke(b, 2.0, st(1.2, c(200)));
            }
            p.rect_filled(
                Rect::from_min_size(pos2(s.left() + 8.0, s.bottom() - 14.0), vec2(s.width() * 0.5, 4.0)),
                0.0,
                c(160),
            );
            let tx = s.center().x + (t * 1.7).sin() * s.width() * 0.3;
            let ty = s.center().y + (t * 2.3).cos() * s.height() * 0.2;
            if hov > 0.01 {
                p.circle_stroke(pos2(tx, ty), 6.0 + 4.0 * (t * 6.0).sin().abs(), st(1.2, c(200)));
                p.circle_filled(pos2(tx, ty), 2.5, c(255));
            } else {
                p.circle_filled(pos2(tx, ty), 2.0, c(110));
            }
        }
        Icon::Battery { mah } => {
            let r = fit(rect, 1.9, 22.0);
            let body = Rect::from_min_max(r.min, pos2(r.right() - 9.0, r.max.y));
            let nub = Rect::from_center_size(
                pos2(r.right() - 4.0, r.center().y),
                vec2(8.0, r.height() * 0.34),
            );
            p.rect_stroke(body, 6.0, st(1.8, c(235)));
            p.rect_filled(nub, 2.0, c(235));
            let base = ((mah as f32 - 200.0) / 700.0).clamp(0.1, 1.0);
            let pulse = 0.5 + 0.5 * (t * 2.2).sin();
            let level = base + (1.0 - base) * pulse * hov;
            let inner = body.shrink(5.0);
            let fill = Rect::from_min_size(inner.min, vec2(inner.width() * level, inner.height()));
            p.rect_filled(fill, 3.0, c((120.0 + 120.0 * level) as u8));
            for k in 1..5 {
                let x = inner.left() + inner.width() * k as f32 / 5.0;
                p.line_segment([pos2(x, inner.top()), pos2(x, inner.bottom())], st(1.2, c(12)));
            }
            p.text(
                pos2(r.center().x, r.bottom() + 14.0),
                Align2::CENTER_CENTER,
                format!("3.7V · {mah}mAh"),
                FontId::monospace(12.0),
                c(190),
            );
        }
        Icon::Pcb => {
            let r = fit(rect, 1.4, 18.0);
            p.rect_filled(r, 4.0, c(10));
            p.rect_stroke(r, 4.0, st(1.6, c(235)));
            let q = |x: f32, y: f32| pos2(r.left() + r.width() * x, r.top() + r.height() * y);
            let traces: [Vec<Pos2>; 3] = [
                vec![q(0.1, 0.22), q(0.4, 0.22), q(0.4, 0.5), q(0.9, 0.5)],
                vec![q(0.1, 0.5), q(0.25, 0.5), q(0.25, 0.82), q(0.9, 0.82)],
                vec![q(0.6, 0.14), q(0.6, 0.34), q(0.9, 0.34)],
            ];
            for tr in &traces {
                for w in tr.windows(2) {
                    p.line_segment([w[0], w[1]], st(1.8, c(170)));
                }
                p.circle_stroke(tr[0], 3.5, st(1.4, c(230)));
                p.circle_filled(*tr.last().unwrap(), 2.5, c(230));
            }
            let ic = Rect::from_center_size(
                q(0.62, 0.67),
                vec2(r.width() * 0.2, r.height() * 0.2),
            );
            p.rect_stroke(ic, 1.0, st(1.2, c(210)));
            for i in 0..4 {
                let x = ic.left() + ic.width() * (i as f32 + 0.5) / 4.0;
                p.rect_filled(Rect::from_center_size(pos2(x, ic.top() - 2.0), vec2(3.0, 3.0)), 0.0, c(200));
                p.rect_filled(Rect::from_center_size(pos2(x, ic.bottom() + 2.0), vec2(3.0, 3.0)), 0.0, c(200));
            }
            if hov > 0.01 {
                for (k, tr) in traces.iter().enumerate() {
                    let pos = along(tr, t * 0.4 + k as f32 * 0.33);
                    p.circle_stroke(pos, 7.0, st(1.0, c(110)));
                    p.circle_filled(pos, 3.5, c(255));
                }
            }
        }
        Icon::Perf => {
            let r = fit(rect, 1.4, 18.0);
            p.rect_filled(r, 4.0, c(10));
            p.rect_stroke(r, 4.0, st(1.6, c(235)));
            let (cols, rows) = (10usize, 6usize);
            let active = if hov > 0.01 {
                ((t * 7.0) as usize) % cols
            } else {
                usize::MAX
            };
            for ri in 0..rows {
                for ci in 0..cols {
                    let pos = pos2(
                        r.left() + r.width() * (ci as f32 + 0.5) / cols as f32,
                        r.top() + r.height() * (ri as f32 + 0.5) / rows as f32,
                    );
                    let solder = (ci * 3 + ri * 5) % 11 == 0;
                    let on = ci == active;
                    if solder {
                        p.circle_filled(pos, 3.6, c(235));
                    } else {
                        p.circle_stroke(pos, 2.2, st(1.0, if on { c(255) } else { c(110) }));
                        if on {
                            p.circle_filled(pos, 1.6, c(255));
                        }
                    }
                }
            }
        }
    }
}
