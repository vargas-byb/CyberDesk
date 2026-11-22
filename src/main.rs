#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auth;
mod data;
mod icons;
mod util;
mod widgets;

use data::{Cat, Item, Level, CATS, ITEMS};
use eframe::egui::emath::easing;
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, ColorImage, Id, LayerId, Order, Painter, Pos2, Rect,
    RichText, Sense, TextureHandle, TextureOptions,
};
use include_dir::{include_dir, Dir};
use std::f32::consts::PI;
use std::time::Duration;
use util::*;
use widgets::*;

static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/items");
const LOGO_PNG: &[u8] = include_bytes!("../assets/icon/icon.png");
const VERSION: &str = env!("CARGO_PKG_VERSION");

const CARD_W: f32 = 224.0;
const CARD_H: f32 = 244.0;
const ICON_H: f32 = 152.0;
const GAP: f32 = 18.0;

// ───────────────────────────── Imagens ─────────────────────────────

fn slug(name: &str) -> String {
    let mut s = String::new();
    for ch in name.chars() {
        let ch = match ch.to_lowercase().next().unwrap_or(ch) {
            'á' | 'à' | 'â' | 'ã' => 'a',
            'é' | 'ê' => 'e',
            'í' => 'i',
            'ó' | 'ô' | 'õ' => 'o',
            'ú' => 'u',
            'ç' => 'c',
            c => c,
        };
        if ch.is_ascii_alphanumeric() {
            s.push(ch.to_ascii_lowercase());
        } else if !s.ends_with('-') {
            s.push('-');
        }
    }
    s.trim_matches('-').to_string()
}

/// Minúsculas e sem acento, para a busca achar "modulo" em "Módulo".
fn norm(s: &str) -> String {
    s.chars()
        .map(|ch| match ch.to_lowercase().next().unwrap_or(ch) {
            'á' | 'à' | 'â' | 'ã' => 'a',
            'é' | 'ê' => 'e',
            'í' => 'i',
            'ó' | 'ô' | 'õ' => 'o',
            'ú' => 'u',
            'ç' => 'c',
            c => c,
        })
        .collect()
}

fn find_image(slug: &str) -> Option<&'static [u8]> {
    for ext in ["png", "jpg", "jpeg", "webp"] {
        if let Some(f) = ASSETS.get_file(format!("{slug}.{ext}")) {
            return Some(f.contents());
        }
    }
    None
}

fn make_tex(
    ctx: &egui::Context,
    name: &str,
    bytes: &[u8],
    max: u32,
    square: bool,
) -> Option<TextureHandle> {
    let mut img = image::load_from_memory(bytes).ok()?;
    if square {
        let (w, h) = (img.width(), img.height());
        let s = w.min(h).max(1);
        img = img.crop_imm((w - s) / 2, (h - s) / 2, s, s);
    }
    let img = img.thumbnail(max, max).to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    Some(ctx.load_texture(
        name,
        ColorImage::from_rgba_unmultiplied(size, img.as_raw()),
        TextureOptions::LINEAR,
    ))
}

fn draw_logo(p: &Painter, tex: &TextureHandle, rect: Rect, fade: f32) {
    p.image(
        tex.id(),
        rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        gray(255, fade),
    );
}

fn paint_img(p: &Painter, area: Rect, tex: &TextureHandle, e: f32, hov: f32, par: egui::Vec2) {
    let panel = area.shrink2(vec2(10.0, 10.0));
    p.rect_filled(panel, 8.0, gray(244, e));
    let s = tex.size_vec2();
    let inner = panel.shrink(6.0);
    let k = (inner.width() / s.x).min(inner.height() / s.y) * (1.0 + 0.07 * hov);
    let r = Rect::from_center_size(inner.center(), s * k).translate(par);
    p.with_clip_rect(panel).image(
        tex.id(),
        r,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        gray(255, e),
    );
}

// ───────────────────────────── Cards ─────────────────────────────

fn level_bars(p: &Painter, pos: Pos2, level: Level, color: Color32, fade: f32) {
    for i in 0..3 {
        let on = i < level.bars();
        let r = Rect::from_min_size(pos + vec2(i as f32 * 8.0, -3.0), vec2(6.0, 6.0));
        p.rect_filled(
            r,
            1.5,
            if on { fc(color, fade) } else { fc(LINE, fade) },
        );
    }
}

/// Estado visual de um card (tudo de 0 a 1, animado por quem chama).
#[derive(Clone, Copy, Default)]
struct CardFx {
    /// mouse sobre o card
    hov: f32,
    /// varredura de brilho sobre a imagem
    shine: f32,
    /// mouse sobre a estrela
    star_hov: f32,
    /// "pulo" da estrela ao favoritar
    star_pop: f32,
}

#[allow(clippy::too_many_arguments)]
fn paint_card(
    p: &Painter,
    r: Rect,
    it: &Item,
    time: f64,
    e: f32,
    tex: Option<&TextureHandle>,
    fav: bool,
    fx: CardFx,
    mouse: Option<Pos2>,
) {
    let CardFx { hov, shine, star_hov, star_pop } = fx;
    let col = it.cat.color();

    // sombra suave que cresce ao passar o mouse
    if hov > 0.01 {
        for i in 0..4 {
            let g = (i + 1) as f32;
            p.rect_filled(
                r.expand(g * 3.0).translate(vec2(0.0, 6.0 + g * 1.5)),
                14.0 + g * 3.0,
                Color32::from_black_alpha((15.0 * hov * e) as u8),
            );
        }
    }
    p.rect_filled(r, 12.0, fc(mix(PANEL, PANEL2, hov), e));
    // brilho que segue o mouse
    if let (Some(m), true) = (mouse, hov > 0.01) {
        let pc = p.with_clip_rect(r);
        glow(&pc, m, 120.0, col, 0.16 * hov * e);
    }
    p.rect_stroke(
        r,
        12.0,
        st(1.0 + 0.6 * hov, fc(mix(LINE, col, hov), e)),
    );
    // barra de destaque no topo: cresce a partir do centro
    let bw = 34.0 + (r.width() - 70.0) * hov;
    p.rect_filled(
        Rect::from_center_size(pos2(r.center().x, r.top() + 0.5), vec2(bw, 2.5)),
        1.2,
        fc(col, e * (0.35 + 0.65 * hov)),
    );

    // cantos estilo "mira"
    let bl = 8.0 + 6.0 * hov;
    let tick = st(1.5, fc(mix(DIM, col, hov), e));
    brackets(p, r.shrink(6.0), bl, tick);

    // imagem (com parallax: acompanha o mouse em sentido contrário)
    let area = Rect::from_min_size(r.min, vec2(CARD_W, ICON_H));
    let par = match (mouse, hov > 0.01) {
        (Some(m), true) => {
            let d = (m - r.center()) * -0.035;
            vec2(d.x.clamp(-5.0, 5.0), d.y.clamp(-4.0, 4.0)) * hov
        }
        _ => vec2(0.0, 0.0),
    };
    let panel = area.shrink2(vec2(10.0, 10.0));
    match tex {
        Some(t) => paint_img(p, area, t, e, hov, par),
        None => {
            p.rect_filled(panel, 8.0, fc(Color32::from_rgb(6, 9, 13), e));
            icons::draw_icon(&p.with_clip_rect(panel), panel, it.icon, hov, time, e);
        }
    }
    sweep(p, panel, shine, (std::f32::consts::PI * shine).sin() * e);

    // etiqueta da categoria sobre a imagem
    let lab = it.cat.label();
    let lw = p
        .layout_no_wrap(lab.to_string(), mono(9.5), Color32::WHITE)
        .size()
        .x;
    let badge = Rect::from_min_size(r.left_top() + vec2(18.0, 18.0), vec2(lw + 22.0, 20.0));
    p.rect_filled(badge, 10.0, Color32::from_black_alpha((190.0 * e) as u8));
    p.circle_filled(pos2(badge.left() + 10.0, badge.center().y), 3.0, fc(col, e));
    p.text(
        pos2(badge.left() + 18.0, badge.center().y + 0.5),
        Align2::LEFT_CENTER,
        lab,
        mono(9.5),
        gray(230, e),
    );

    // estrela de favorito (com "pulo" ao favoritar)
    let sc = pos2(r.right() - 29.0, r.top() + 28.0);
    if fav || hov > 0.01 || star_pop > 0.01 {
        let a = if fav { 1.0 } else { hov.max(star_pop) };
        let pop = 5.0 * star_pop;
        p.circle_filled(
            sc,
            14.0 + 1.5 * star_hov + pop,
            Color32::from_black_alpha((200.0 * a * e) as u8),
        );
        let c = if fav {
            fc(GOLD, e)
        } else {
            fc(mix(Color32::from_gray(170), GOLD, star_hov), a * e)
        };
        star_icon(p, sc, 7.5 + 1.5 * star_hov + pop * 0.8, fav, c);
    }

    // texto
    let y0 = r.top() + ICON_H;
    p.line_segment(
        [pos2(r.left() + 16.0, y0), pos2(r.right() - 16.0, y0)],
        st(1.0, fc(LINE, e)),
    );
    let mut fs = 16.0;
    let nw = p
        .layout_no_wrap(it.name.to_string(), sans(16.0), Color32::WHITE)
        .size()
        .x;
    if nw > CARD_W - 36.0 {
        fs = (16.0 * (CARD_W - 36.0) / nw).max(12.0);
    }
    p.text(
        pos2(r.left() + 18.0, y0 + 21.0),
        Align2::LEFT_CENTER,
        it.name,
        sans(fs),
        fc(TEXT, e),
    );
    let tag = ellipsize(p, it.tagline, sans(12.5), CARD_W - 36.0);
    p.text(
        pos2(r.left() + 18.0, y0 + 43.0),
        Align2::LEFT_CENTER,
        tag,
        sans(12.5),
        fc(mix(MUTED, Color32::from_gray(190), hov), e),
    );
    level_bars(p, pos2(r.left() + 18.0, y0 + 69.0), it.level, col, e);
    p.text(
        pos2(r.left() + 18.0 + 28.0, y0 + 69.0),
        Align2::LEFT_CENTER,
        it.level.label(),
        mono(9.5),
        fc(DIM, e),
    );
    if hov > 0.02 {
        p.text(
            pos2(r.right() - 18.0 - 6.0 * (1.0 - hov), y0 + 69.0),
            Align2::RIGHT_CENTER,
            "ABRIR →",
            mono(10.5),
            fc(col, e * hov),
        );
    }
}

/// Desenha um card. Retorna (card clicado, estrela clicada).
#[allow(clippy::too_many_arguments)]
fn card(
    ui: &mut egui::Ui,
    it: &Item,
    idx: usize,
    order: usize,
    t0: f64,
    active: bool,
    tex: Option<&TextureHandle>,
    fav: bool,
    laid: &mut Vec<(usize, Rect)>,
) -> (bool, bool) {
    let time = ui.input(|i| i.time);
    let k = (((time - t0) - order as f64 * 0.035) / 0.5).clamp(0.0, 1.0) as f32;
    let e = easing::cubic_out(k);
    let (rect, resp) = ui.allocate_exact_size(vec2(CARD_W, CARD_H), Sense::click());
    laid.push((idx, rect));
    if k <= 0.0 || active {
        return (false, false);
    }
    let ctx = ui.ctx().clone();
    let hov = ctx.animate_bool_with_time(resp.id.with("hov"), resp.hovered(), 0.18);
    let shine = ctx.animate_bool_with_time(resp.id.with("shine"), resp.hovered(), 0.7);
    let r = rect.translate(vec2(0.0, -5.0 * hov + 26.0 * (1.0 - e)));
    let star_rect = Rect::from_center_size(pos2(r.right() - 29.0, r.top() + 28.0), vec2(30.0, 30.0));
    let sresp = ui.interact(star_rect, resp.id.with("star"), Sense::click());
    let star_hov = ctx.animate_bool_with_time(resp.id.with("star_h"), sresp.hovered(), 0.12);
    // 0 -> 1 quando vira favorito (e de volta); o "pulo" só existe durante a transição
    let fav_t = ctx.animate_bool_with_time(resp.id.with("fav_t"), fav, 0.4);
    let star_pop = if fav_t > 0.0 && fav_t < 1.0 {
        (std::f32::consts::PI * fav_t).sin()
    } else {
        0.0
    };
    if resp.hovered() || sresp.hovered() {
        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let mouse = ui.input(|i| i.pointer.hover_pos());
    let fx = CardFx { hov, shine, star_hov, star_pop };
    paint_card(ui.painter(), r, it, time, e, tex, fav, fx, mouse);
    if sresp.hovered() {
        sresp.clone().on_hover_text(if fav {
            "Remover dos favoritos (F)"
        } else {
            "Adicionar aos favoritos (F)"
        });
    }
    (resp.clicked() && !sresp.clicked(), sresp.clicked())
}

// ───────────────────────────── App ─────────────────────────────

#[derive(PartialEq, Clone, Copy)]
enum Screen {
    Splash,
    Login,
    Catalog,
}

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Login,
    Register,
}

#[derive(PartialEq, Clone, Copy)]
enum Sort {
    Default,
    Az,
    Level,
}

impl Sort {
    fn next(self) -> Self {
        match self {
            Sort::Default => Sort::Az,
            Sort::Az => Sort::Level,
            Sort::Level => Sort::Default,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Sort::Default => "ORDEM · PADRÃO",
            Sort::Az => "ORDEM · A–Z",
            Sort::Level => "ORDEM · NÍVEL",
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Filter {
    All,
    Favs,
    Cat(Cat),
}

struct Sel {
    idx: usize,
    t0: f64,
}

struct Toast {
    msg: String,
    color: Color32,
    t0: f64,
}

/// Faíscas que saem da estrela ao favoritar um item.
struct Burst {
    pos: Pos2,
    color: Color32,
    t0: f64,
}

/// Login aceito: mostra a animação de "acesso concedido" antes de abrir o catálogo.
struct Pending {
    name: String,
    greeting: &'static str,
    t0: f64,
}

struct CyberDesk {
    screen: Screen,
    boot_t0: f64,
    restored: Option<String>,
    logo: Option<TextureHandle>,

    store: auth::Store,
    user: Option<String>,
    mode: Mode,
    f_user: String,
    f_pass: String,
    f_pass2: String,
    remember: bool,
    show_pw: bool,
    focus_user: bool,
    err: Option<(String, f64)>,
    fails: u32,
    locked_until: f64,
    pending_avatar: Option<(Vec<u8>, TextureHandle)>,
    avatar: Option<TextureHandle>,
    menu_open: bool,
    login_t0: f64,
    focus_pw: bool,
    pending: Option<Pending>,
    bursts: Vec<Burst>,
    scroll_y: f32,
    scroll_top: bool,

    filter: Filter,
    sort: Sort,
    query: String,
    last_key: (Filter, Sort, String),
    grid_t0: f64,
    sel: Option<Sel>,
    open: bool,
    imgs: Vec<Option<TextureHandle>>,
    blobs: Vec<String>,
    rects: Vec<Option<Rect>>,
    visible: Vec<usize>,
    scroll_to: Option<usize>,
    favs: Vec<String>,
    toasts: Vec<Toast>,
}

fn typed_title(t: f64) -> String {
    let full = "CYBERDESK";
    let n = ((t.max(0.0) * 12.0) as usize).min(full.len());
    let cursor = if n < full.len() || (t * 2.0) as i64 % 2 == 0 {
        "_"
    } else {
        ""
    };
    format!("{}{}", &full[..n], cursor)
}

impl CyberDesk {
    fn new(ctx: &egui::Context) -> Self {
        let imgs = ITEMS
            .iter()
            .map(|it| {
                let s = slug(it.name);
                find_image(&s).and_then(|b| make_tex(ctx, &format!("item-{s}"), b, 512, false))
            })
            .collect();
        let blobs = ITEMS
            .iter()
            .map(|it| {
                norm(&format!(
                    "{} {} {} {} {}",
                    it.name,
                    it.cat.label(),
                    it.chip,
                    it.desc,
                    it.uses.join(" ")
                ))
            })
            .collect();
        let mut store = auth::Store::load();
        // sessão salva: se ainda for válida, a pessoa já entra logada
        let restored = store.restore_session();
        let last_user = store.last_user().unwrap_or_default();
        let prefilled = !last_user.is_empty();
        Self {
            screen: Screen::Splash,
            boot_t0: 0.0,
            restored,
            logo: make_tex(ctx, "logo", LOGO_PNG, 256, false),
            store,
            user: None,
            mode: Mode::Login,
            f_user: last_user,
            f_pass: String::new(),
            f_pass2: String::new(),
            remember: true,
            show_pw: false,
            focus_user: !prefilled,
            focus_pw: prefilled,
            pending: None,
            bursts: Vec::new(),
            scroll_y: 0.0,
            scroll_top: false,
            err: None,
            fails: 0,
            locked_until: 0.0,
            pending_avatar: None,
            avatar: None,
            menu_open: false,
            login_t0: 0.0,
            filter: Filter::All,
            sort: Sort::Default,
            query: String::new(),
            last_key: (Filter::All, Sort::Default, String::new()),
            grid_t0: 0.0,
            sel: None,
            open: false,
            imgs,
            blobs,
            rects: vec![None; ITEMS.len()],
            visible: Vec::new(),
            scroll_to: None,
            favs: Vec::new(),
            toasts: Vec::new(),
        }
    }

    fn toast(&mut self, msg: impl Into<String>, color: Color32, time: f64) {
        self.toasts.push(Toast {
            msg: msg.into(),
            color,
            t0: time,
        });
        if self.toasts.len() > 3 {
            self.toasts.remove(0);
        }
    }

    fn fail(&mut self, msg: &str, time: f64) {
        self.err = Some((msg.to_string(), time));
    }

    fn is_fav(&self, name: &str) -> bool {
        self.favs.iter().any(|f| f == name)
    }

    fn toggle_fav(&mut self, idx: usize, time: f64) {
        let name = ITEMS[idx].name.to_string();
        if let Some(pos) = self.favs.iter().position(|f| *f == name) {
            self.favs.remove(pos);
            self.toast(format!("Removido dos favoritos · {name}"), MUTED, time);
        } else {
            self.favs.push(name.clone());
            self.toast(format!("Adicionado aos favoritos · {name}"), GOLD, time);
            if let Some(Some(r)) = self.rects.get(idx) {
                self.bursts.push(Burst {
                    pos: pos2(r.right() - 29.0, r.top() + 28.0 - 5.0),
                    color: GOLD,
                    t0: time,
                });
            }
        }
        if let Some(u) = &self.user {
            self.store.save_favorites(u, &self.favs);
        }
    }

    fn load_avatar(&mut self, ctx: &egui::Context) {
        self.avatar = self.user.as_ref().and_then(|u| {
            self.store
                .avatar_bytes(u)
                .and_then(|b| make_tex(ctx, &format!("avatar-{u}"), &b, 256, true))
        });
    }

    /// Entra no catálogo (login, cadastro ou sessão restaurada).
    fn enter(&mut self, ctx: &egui::Context, name: String, time: f64, greeting: &str) {
        self.favs = self.store.favorites(&name);
        self.store.set_last_user(&name);
        self.user = Some(name.clone());
        self.pending = None;
        self.screen = Screen::Catalog;
        self.f_user.clear();
        self.f_pass.clear();
        self.f_pass2.clear();
        self.err = None;
        self.fails = 0;
        self.pending_avatar = None;
        self.mode = Mode::Login;
        self.show_pw = false;
        self.login_t0 = time;
        self.grid_t0 = time;
        self.filter = Filter::All;
        self.query.clear();
        self.load_avatar(ctx);
        self.toast(format!("{greeting}, {name}"), ACCENT, time + 0.5);
    }

    fn apply_avatar(&mut self, ctx: &egui::Context, bytes: Vec<u8>, time: f64) {
        if bytes.len() > 20 * 1024 * 1024 {
            self.fail("Imagem muito grande (máx. 20 MB)", time);
            return;
        }
        if let Some(u) = self.user.clone() {
            match self.store.set_avatar(&u, &bytes) {
                Ok(()) => {
                    self.load_avatar(ctx);
                    self.toast("Foto de perfil atualizada", OK, time);
                }
                Err(e) => self.toast(e, DANGER, time),
            }
        } else if self.screen == Screen::Login && self.mode == Mode::Register {
            match make_tex(ctx, "avatar-pending", &bytes, 256, true) {
                Some(t) => {
                    self.err = None;
                    self.pending_avatar = Some((bytes, t));
                }
                None => self.fail("Imagem inválida", time),
            }
        }
    }

    fn pick_file(&mut self, ctx: &egui::Context, time: f64) {
        let file = rfd::FileDialog::new()
            .add_filter("Imagem", &["png", "jpg", "jpeg", "webp"])
            .pick_file();
        if let Some(path) = file {
            match std::fs::read(&path) {
                Ok(b) => self.apply_avatar(ctx, b, time),
                Err(_) => self.fail("Não foi possível ler o arquivo", time),
            }
        }
    }

    fn submit(&mut self, ctx: &egui::Context, time: f64) {
        if time < self.locked_until {
            let s = (self.locked_until - time).ceil() as i32;
            self.fail(&format!("Muitas tentativas. Aguarde {s}s"), time);
            return;
        }
        let registering = self.mode == Mode::Register;
        let result = if registering {
            if self.f_pass != self.f_pass2 {
                Err("As senhas não coincidem".to_string())
            } else {
                match self.store.register(&self.f_user, &self.f_pass) {
                    Ok(name) => {
                        if let Some((bytes, _)) = &self.pending_avatar {
                            let _ = self.store.set_avatar(&name, bytes);
                        }
                        Ok(name)
                    }
                    Err(e) => Err(e),
                }
            }
        } else {
            self.store.login(&self.f_user, &self.f_pass)
        };
        match result {
            Ok(name) => {
                // "manter conectado": guarda a sessão para a próxima abertura
                self.store.remember(&name, self.remember);
                let greeting = if registering {
                    "Conta criada! Bem-vindo"
                } else {
                    "Bem-vindo de volta"
                };
                let _ = ctx;
                self.pending = Some(Pending { name, greeting, t0: time });
            }
            Err(e) => {
                if !registering {
                    self.fails += 1;
                    if self.fails >= 5 {
                        self.fails = 0;
                        self.locked_until = time + 30.0;
                        self.fail("Muitas tentativas. Aguarde 30s", time);
                        return;
                    }
                }
                self.fail(&e, time);
            }
        }
    }

    fn logout(&mut self, time: f64) {
        if let Some(u) = self.user.take() {
            // sair apaga a sessão salva: na próxima abertura pede login
            self.store.forget(&u);
        }
        self.avatar = None;
        self.menu_open = false;
        self.open = false;
        self.sel = None;
        self.favs.clear();
        self.screen = Screen::Login;
        self.mode = Mode::Login;
        self.err = None;
        self.focus_user = true;
        self.login_t0 = time;
        self.toast("Você saiu da conta", MUTED, time);
    }

    // ───── Login aceito: "acesso concedido" ─────
    fn success_overlay(&mut self, ctx: &egui::Context, time: f64) {
        let Some(pd) = &self.pending else { return };
        let (t, name, greeting) = ((time - pd.t0) as f32, pd.name.clone(), pd.greeting);
        let total = 1.15;
        if t >= total {
            self.enter(ctx, name, time, greeting);
            return;
        }
        let created = greeting.starts_with("Conta");
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let r = ui.max_rect();
                let p = ui.painter();
                let c = r.center() + vec2(0.0, -24.0);
                let e = easing::cubic_out((t / 0.45).clamp(0.0, 1.0));
                let out = easing::cubic_in(((t - (total - 0.3)) / 0.3).clamp(0.0, 1.0));
                let fade = e * (1.0 - out);
                glow(p, c, 240.0, OK, 0.17 * fade);
                // anel que se fecha
                let ring = easing::cubic_in_out((t / 0.6).clamp(0.0, 1.0));
                p.circle_filled(c, 52.0, fc(PANEL, fade));
                arc_smooth(p, c, 52.0, 0.0, 2.0 * PI, st(2.0, fc(LINE, fade)));
                arc_smooth(
                    p,
                    c,
                    52.0,
                    -PI / 2.0,
                    -PI / 2.0 + ring * 2.0 * PI,
                    st(3.5, fc(OK, fade)),
                );
                // o "check" é desenhado em dois traços
                let k = ((t - 0.38) / 0.4).clamp(0.0, 1.0);
                let (a, b, d) = (
                    c + vec2(-17.0, 2.0),
                    c + vec2(-5.0, 14.0),
                    c + vec2(19.0, -13.0),
                );
                let k1 = (k * 2.0).min(1.0);
                p.line_segment([a, a + (b - a) * k1], st(5.0, fc(OK, fade)));
                if k > 0.5 {
                    let k2 = (k - 0.5) * 2.0;
                    p.line_segment([b, b + (d - b) * k2], st(5.0, fc(OK, fade)));
                }
                let ty = ((t - 0.3) / 0.4).clamp(0.0, 1.0);
                let ty = easing::cubic_out(ty) * fade;
                p.text(
                    pos2(c.x, c.y + 96.0 - 8.0 * (1.0 - ty)),
                    Align2::CENTER_CENTER,
                    if created { "CONTA CRIADA" } else { "ACESSO CONCEDIDO" },
                    bold(24.0),
                    fc(TEXT, ty),
                );
                p.text(
                    pos2(c.x, c.y + 128.0 - 8.0 * (1.0 - ty)),
                    Align2::CENTER_CENTER,
                    format!("// {name}"),
                    mono(13.0),
                    fc(OK, ty),
                );
            });
    }

    // ───── Faíscas (favoritar) ─────
    fn draw_fx(&mut self, ctx: &egui::Context, time: f64) {
        self.bursts.retain(|b| time - b.t0 < 0.9);
        if self.bursts.is_empty() {
            return;
        }
        let p = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::new("fx")));
        for (bi, b) in self.bursts.iter().enumerate() {
            let k = ((time - b.t0) / 0.9) as f32;
            let e = easing::cubic_out(k);
            let fade = (1.0 - k).powf(1.3);
            p.circle_stroke(b.pos, 8.0 + 30.0 * e, st(2.0 * (1.0 - k) + 0.5, alpha(b.color, 0.8 * fade)));
            for i in 0..12u32 {
                let ang = i as f32 / 12.0 * 2.0 * PI + hash01(i + bi as u32 * 31) * 0.5;
                let dist = 10.0 + (22.0 + 26.0 * hash01(i * 5 + 9)) * e;
                let q = b.pos + vec2(ang.cos(), ang.sin()) * dist;
                let sz = (1.6 + 2.2 * hash01(i * 3 + 1)) * (1.0 - k * 0.7);
                p.circle_filled(q, sz, alpha(if i % 2 == 0 { b.color } else { TEXT }, fade));
            }
        }
    }

    // ───── Splash ─────
    fn splash(&mut self, ctx: &egui::Context, time: f64) {
        let t = (time - self.boot_t0) as f32;
        let total = 2.1;
        let skip = t > 0.6
            && ctx.input(|i| {
                i.pointer.any_click()
                    || i.events
                        .iter()
                        .any(|e| matches!(e, egui::Event::Key { pressed: true, .. }))
            });
        if t >= total || skip {
            match self.restored.take() {
                Some(name) => self.enter(ctx, name, time, "Sessão restaurada · olá"),
                None => {
                    self.screen = Screen::Login;
                    self.login_t0 = time;
                    self.focus_user = true;
                }
            }
            return;
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let r = ui.max_rect();
                let p = ui.painter();
                let c = r.center() + vec2(0.0, -40.0);
                let e = easing::cubic_out((t / 0.8).clamp(0.0, 1.0));
                let out = easing::cubic_in(((t - (total - 0.35)) / 0.35).clamp(0.0, 1.0));
                let fade = e * (1.0 - out);

                glow(p, c, 260.0, ACCENT, 0.14 * fade);
                // anéis girando
                for ring in 0..3 {
                    let rad = 96.0 + ring as f32 * 16.0;
                    let dir = if ring % 2 == 0 { 1.0 } else { -1.0 };
                    for i in 0..18 {
                        let a0 = i as f32 / 18.0 * 2.0 * PI + t * (0.5 + 0.3 * ring as f32) * dir;
                        arc(
                            p,
                            c,
                            rad * (0.8 + 0.2 * e),
                            a0,
                            a0 + 0.14,
                            st(1.4, alpha(ACCENT, (0.55 - 0.15 * ring as f32) * fade)),
                        );
                    }
                }
                if let Some(logo) = &self.logo {
                    let s = 132.0 * (0.8 + 0.2 * e);
                    let lr = Rect::from_center_size(c, vec2(s, s));
                    // halo atrás do ícone (ele é escuro) + moldura HUD que se abre
                    glow(p, c, 120.0, ACCENT, 0.22 * fade);
                    draw_logo(p, logo, lr, fade);
                    brackets(p, lr.expand(12.0 + 10.0 * (1.0 - e)), 16.0, st(1.8, alpha(ACCENT, 0.85 * fade)));
                    // linha de varredura passando sobre o ícone
                    let sy = lr.top() + lr.height() * ((t * 0.9) % 1.0);
                    let sc = p.with_clip_rect(lr);
                    sc.line_segment(
                        [pos2(lr.left(), sy), pos2(lr.right(), sy)],
                        st(1.5, alpha(ACCENT, 0.55 * fade)),
                    );
                    glow(&sc, pos2(c.x, sy), 60.0, ACCENT, 0.12 * fade);
                }
                p.text(
                    pos2(c.x, c.y + 150.0),
                    Align2::CENTER_CENTER,
                    typed_title((t - 0.35) as f64),
                    bold(34.0),
                    fc(TEXT, fade),
                );
                let status: String = if t < 0.7 {
                    "inicializando núcleo…".into()
                } else if t < 1.3 {
                    "carregando catálogo de hardware…".into()
                } else if t < 1.8 {
                    if self.restored.is_some() {
                        "sessão salva encontrada…".into()
                    } else {
                        "preparando o acesso…".into()
                    }
                } else if let Some(n) = &self.restored {
                    format!("pronto · olá, {n}")
                } else {
                    "pronto".into()
                };
                p.text(
                    pos2(c.x, c.y + 190.0),
                    Align2::CENTER_CENTER,
                    format!("// {status}"),
                    mono(12.0),
                    fc(MUTED, fade),
                );
                let bw = 280.0;
                let bar = Rect::from_center_size(pos2(c.x, c.y + 218.0), vec2(bw, 4.0));
                p.rect_filled(bar, 2.0, fc(LINE, fade));
                let k = easing::cubic_in_out((t / (total - 0.2)).clamp(0.0, 1.0));
                let fill = Rect::from_min_size(bar.min, vec2(bw * k, 4.0));
                p.rect_filled(fill, 2.0, fc(ACCENT, fade));
                glow(p, fill.right_center(), 16.0, ACCENT, 0.5 * fade);
                p.text(
                    pos2(c.x, r.bottom() - 28.0),
                    Align2::CENTER_CENTER,
                    format!("v{VERSION} · toque para pular"),
                    mono(10.5),
                    fc(DIM, fade),
                );
            });
    }

    // ───── Tela de login / cadastro ─────
    fn login_screen(&mut self, ctx: &egui::Context, time: f64) {
        let e = easing::cubic_out(((time - self.login_t0) / 0.9).clamp(0.0, 1.0) as f32);
        let shake = self
            .err
            .as_ref()
            .map(|(_, t0)| {
                let k = ((time - *t0) / 0.5) as f32;
                if k < 1.0 {
                    (k * 40.0).sin() * 10.0 * (1.0 - k)
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);
        let reg = self.mode == Mode::Register;
        // Enter avança para o próximo campo; só envia quando já está tudo preenchido
        let mut do_submit = false;
        // o foco é pedido só depois de desenhar os campos (o campo atual o "devolve" ao receber Enter)
        let mut next_focus: Option<Id> = None;
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            let (fu, fp) = ctx.memory(|m| (m.has_focus(Id::new("f_user")), m.has_focus(Id::new("f_pass"))));
            if fu && self.f_pass.is_empty() {
                next_focus = Some(Id::new("f_pass"));
            } else if reg && fp && self.f_pass2.is_empty() {
                next_focus = Some(Id::new("f_pass2"));
            } else {
                do_submit = true;
            }
        }
        let mut pick = false;
        let mut switch = false;
        let focus_user = std::mem::take(&mut self.focus_user);
        let focus_pw = std::mem::take(&mut self.focus_pw);

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let w = 372.0;
                let area = ui.max_rect();
                // em janelas largas o formulário vai para a direita e a marca fica à esquerda
                let wide = area.width() >= 1040.0;
                // (em modo largo o cartão não repete o logo, então fica mais baixo)
                let h = (if reg { 626.0 } else { 440.0 }) - if wide { 92.0 } else { 0.0 };
                let cx = if wide {
                    area.left() + area.width() * 0.70
                } else {
                    area.center().x
                };
                let rect = Rect::from_center_size(
                    pos2(cx, area.center().y) + vec2(shake, (1.0 - e) * 30.0),
                    vec2(w, h),
                );

                circuit_traces(ui.painter(), area, time, e);
                if wide {
                    let hero_right = rect.left() - 90.0;
                    self.hero(ui.painter(), area, hero_right, time, e);
                }

                // painel do cartão de login
                let card = rect.expand2(vec2(40.0, 30.0));
                {
                    let p = ui.painter();
                    glow(p, card.center(), card.width() * 1.1, ACCENT, 0.06 * e);
                    p.rect_filled(card, 18.0, fc(PANEL, e));
                    p.rect_stroke(card, 18.0, st(1.0, fc(LINE, e)));
                    let top = Rect::from_min_size(card.left_top() + vec2(30.0, 0.0), vec2(card.width() - 60.0, 2.0));
                    hgrad(p, top, alpha(ACCENT, 0.0), alpha(ACCENT, 0.9 * e));
                }

                ui.allocate_new_ui(
                    egui::UiBuilder::new()
                        .max_rect(rect)
                        .layout(egui::Layout::top_down(Align::Center)),
                    |ui| {
                        if wide {
                            // a marca já aparece no painel da esquerda: aqui só o título da ação
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(if reg { "Criar conta" } else { "Entrar" })
                                    .font(bold(32.0))
                                    .color(fc(TEXT, e)),
                            );
                        } else {
                            // logo com anel
                            let (lr, _) = ui.allocate_exact_size(vec2(96.0, 96.0), Sense::hover());
                            {
                                let p = ui.painter();
                                for i in 0..22 {
                                    let a0 = i as f32 / 22.0 * 2.0 * PI + time as f32 * 0.5;
                                    arc(p, lr.center(), 47.0, a0, a0 + 0.12, st(1.4, alpha(ACCENT, 0.6 * e)));
                                }
                                if let Some(logo) = &self.logo {
                                    draw_logo(
                                        p,
                                        logo,
                                        Rect::from_center_size(lr.center(), vec2(70.0, 70.0)),
                                        e,
                                    );
                                }
                            }
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(typed_title(time - self.login_t0))
                                    .font(bold(30.0))
                                    .color(fc(TEXT, e)),
                            );
                        }
                        ui.label(
                            RichText::new(if reg {
                                "Crie sua conta local em segundos"
                            } else if !self.f_user.is_empty() && focus_pw {
                                "Bem-vindo de volta · digite sua senha"
                            } else {
                                "Entre para acessar o catálogo"
                            })
                            .font(sans(13.5))
                            .color(fc(MUTED, e)),
                        );
                        ui.add_space(18.0);

                        if reg {
                            let (ar, resp) =
                                ui.allocate_exact_size(vec2(84.0, 84.0), Sense::click());
                            let hv = ui
                                .ctx()
                                .animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.2);
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            let ctr = ar.center();
                            let p = ui.painter();
                            match &self.pending_avatar {
                                Some((_, tex)) => draw_avatar(p, ctr, 34.0, Some(tex), "", e),
                                None => {
                                    p.circle_filled(ctr, 34.0, fc(PANEL2, e));
                                    p.line_segment(
                                        [ctr + vec2(-9.0, 0.0), ctr + vec2(9.0, 0.0)],
                                        st(2.0, fc(MUTED, e)),
                                    );
                                    p.line_segment(
                                        [ctr + vec2(0.0, -9.0), ctr + vec2(0.0, 9.0)],
                                        st(2.0, fc(MUTED, e)),
                                    );
                                }
                            }
                            for i in 0..24 {
                                let a0 = i as f32 / 24.0 * 2.0 * PI + time as f32 * (0.3 + 1.6 * hv);
                                arc(p, ctr, 39.0 + 2.0 * hv, a0, a0 + 0.15, st(1.4, alpha(ACCENT, 0.7 * e)));
                            }
                            if resp.clicked() {
                                pick = true;
                            }
                            ui.label(
                                RichText::new("Foto de perfil (opcional) · clique ou arraste")
                                    .font(sans(11.5))
                                    .color(fc(DIM, e)),
                            );
                            ui.add_space(12.0);
                        }

                        // campos
                        let id_user = Id::new("f_user");
                        if focus_user {
                            ui.memory_mut(|m| m.request_focus(id_user));
                        }
                        field(ui, id_user, &mut self.f_user, "Usuário", false, w, 46.0, e, 4.0, 4.0);
                        ui.add_space(10.0);

                        let id_pw = Id::new("f_pass");
                        if focus_pw {
                            ui.memory_mut(|m| m.request_focus(id_pw));
                        }
                        let pw_hint = if reg { "Senha (mín. 6 caracteres)" } else { "Senha" };
                        let (pr, _) = field(
                            ui,
                            id_pw,
                            &mut self.f_pass,
                            pw_hint,
                            !self.show_pw,
                            w,
                            46.0,
                            e,
                            4.0,
                            40.0,
                        );
                        let eye = Rect::from_center_size(pos2(pr.right() - 22.0, pr.center().y), vec2(32.0, 32.0));
                        let er = ui.interact(eye, id_pw.with("eye"), Sense::click());
                        if er.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        eye_icon(
                            ui.painter(),
                            eye.center(),
                            self.show_pw,
                            fc(if er.hovered() { TEXT } else { MUTED }, e),
                        );
                        if er.clicked() {
                            self.show_pw = !self.show_pw;
                        }

                        if reg {
                            ui.add_space(10.0);
                            field(
                                ui,
                                Id::new("f_pass2"),
                                &mut self.f_pass2,
                                "Confirmar senha",
                                !self.show_pw,
                                w,
                                46.0,
                                e,
                                4.0,
                                4.0,
                            );
                            ui.add_space(10.0);
                            // força da senha
                            let score = password_score(&self.f_pass);
                            let (sr, _) = ui.allocate_exact_size(vec2(w, 16.0), Sense::hover());
                            let p = ui.painter();
                            let seg = (w - 90.0 - 12.0) / 4.0;
                            let col = [DIM, DANGER, GOLD, ACCENT, OK][score];
                            for i in 0..4 {
                                let on = i < score;
                                let a = ctx.animate_bool_with_time(Id::new(("pw_seg", i)), on, 0.2);
                                let rr = Rect::from_min_size(
                                    pos2(sr.left() + i as f32 * (seg + 4.0), sr.center().y - 2.0),
                                    vec2(seg, 4.0),
                                );
                                p.rect_filled(rr, 2.0, fc(mix(LINE, col, a), e));
                            }
                            let lab = ["", "fraca", "ok", "boa", "forte"][score];
                            p.text(
                                pos2(sr.right(), sr.center().y),
                                Align2::RIGHT_CENTER,
                                lab,
                                mono(11.0),
                                fc(col, e),
                            );
                        }

                        ui.add_space(10.0);
                        // manter conectado
                        let (cr, cresp) = ui.allocate_exact_size(vec2(w, 26.0), Sense::click());
                        if cresp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if cresp.clicked() {
                            self.remember = !self.remember;
                        }
                        let on = ctx.animate_bool_with_time(Id::new("remember_t"), self.remember, 0.15);
                        {
                            let p = ui.painter();
                            let bx = Rect::from_min_size(cr.left_top() + vec2(2.0, 4.0), vec2(18.0, 18.0));
                            p.rect_filled(bx, 5.0, fc(mix(PANEL2, ACCENT, on), e));
                            p.rect_stroke(bx, 5.0, st(1.2, fc(mix(LINE, ACCENT, on.max(cresp.hovered() as u8 as f32 * 0.6)), e)));
                            if on > 0.01 {
                                let k = on;
                                let a = bx.left_top() + vec2(4.5, 9.5);
                                let b = bx.left_top() + vec2(7.8, 12.8);
                                let c2 = bx.left_top() + vec2(13.5, 5.5);
                                p.line_segment([a, a + (b - a) * k], st(2.0, fc(Color32::from_rgb(2, 20, 26), e)));
                                if k > 0.5 {
                                    p.line_segment([b, b + (c2 - b) * ((k - 0.5) * 2.0)], st(2.0, fc(Color32::from_rgb(2, 20, 26), e)));
                                }
                            }
                            p.text(
                                pos2(bx.right() + 10.0, cr.center().y),
                                Align2::LEFT_CENTER,
                                "Manter conectado neste computador",
                                sans(13.0),
                                fc(mix(MUTED, TEXT, on), e),
                            );
                        }

                        ui.add_space(6.0);
                        // mensagem de erro / dica
                        let (mr, _) = ui.allocate_exact_size(vec2(w, 20.0), Sense::hover());
                        {
                            let p = ui.painter();
                            match &self.err {
                                Some((m, _)) => {
                                    p.circle_filled(pos2(mr.left() + 5.0, mr.center().y), 3.5, fc(DANGER, e));
                                    p.text(
                                        pos2(mr.left() + 16.0, mr.center().y),
                                        Align2::LEFT_CENTER,
                                        m,
                                        sans(12.5),
                                        fc(DANGER, e),
                                    );
                                }
                                None => {
                                    let hint = if reg {
                                        "Seus dados ficam só neste PC (senha protegida com Argon2)"
                                    } else {
                                        "Seus dados ficam só neste PC · senha protegida com Argon2"
                                    };
                                    p.text(
                                        mr.left_center(),
                                        Align2::LEFT_CENTER,
                                        hint,
                                        sans(11.5),
                                        fc(DIM, e),
                                    );
                                }
                            }
                        }
                        ui.add_space(8.0);

                        let label = if reg { "CRIAR CONTA" } else { "ENTRAR" };
                        if big_button(ui, label, w, e) {
                            do_submit = true;
                        }
                        ui.add_space(14.0);
                        let toggle = if reg {
                            "Já tenho conta · entrar"
                        } else {
                            "Não tenho conta · criar agora"
                        };
                        let r = ui.add(
                            egui::Label::new(
                                RichText::new(toggle)
                                    .font(sans(13.0))
                                    .color(fc(ACCENT, e))
                                    .underline(),
                            )
                            .sense(Sense::click()),
                        );
                        if r.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if r.clicked() {
                            switch = true;
                        }
                    },
                );
                // rodapé
                ui.painter().text(
                    pos2(area.center().x, area.bottom() - 22.0),
                    Align2::CENTER_CENTER,
                    format!("CyberDesk v{VERSION}"),
                    mono(10.5),
                    fc(DIM, e),
                );
            });

        self.drop_overlay(ctx, self.mode == Mode::Register);

        if pick {
            self.pick_file(ctx, time);
        }
        if switch {
            self.mode = if self.mode == Mode::Login {
                Mode::Register
            } else {
                Mode::Login
            };
            self.err = None;
            self.f_pass.clear();
            self.f_pass2.clear();
            self.focus_user = true;
        }
        if let Some(id) = next_focus {
            ctx.memory_mut(|m| m.request_focus(id));
        }
        if do_submit {
            self.submit(ctx, time);
        }
    }

    /// Painel de marca (lado esquerdo do login em janelas largas).
    fn hero(&self, p: &Painter, area: Rect, right: f32, time: f64, e: f32) {
        let left = area.left() + (area.width() * 0.07).max(56.0);
        let width = (right - left).clamp(280.0, 560.0);
        let t = (time - self.login_t0) as f32;
        let top = area.center().y - 215.0;

        // logo com anel girando
        let lc = pos2(left + 46.0, top + 46.0);
        glow(p, lc, 110.0, ACCENT, 0.12 * e);
        for i in 0..26 {
            let a0 = i as f32 / 26.0 * 2.0 * PI - time as f32 * 0.45;
            arc(p, lc, 44.0, a0, a0 + 0.11, st(1.5, alpha(ACCENT, 0.6 * e)));
        }
        if let Some(logo) = &self.logo {
            draw_logo(p, logo, Rect::from_center_size(lc, vec2(64.0, 64.0)), e);
        }

        // título gigante digitado
        let k = easing::cubic_out((t / 0.7).clamp(0.0, 1.0));
        p.text(
            pos2(left + 104.0, top + 34.0 + 8.0 * (1.0 - k)),
            Align2::LEFT_CENTER,
            typed_title(time - self.login_t0),
            bold(46.0),
            fc(TEXT, e),
        );
        p.text(
            pos2(left + 106.0, top + 70.0),
            Align2::LEFT_CENTER,
            "HARDWARE · CIBERSEGURANÇA · ESP32 · ARDUINO",
            mono(11.5),
            fc(ACCENT, e * 0.9),
        );

        // chamada principal
        let galley = p.layout(
            "O catálogo de hardware para quem estuda segurança e constrói com ESP32 e Arduino.".to_string(),
            sans(19.0),
            fc(Color32::from_gray(205), e),
            width,
        );
        let ky = easing::cubic_out(((t - 0.15) / 0.6).clamp(0.0, 1.0));
        p.galley(pos2(left, top + 118.0 + 10.0 * (1.0 - ky)), galley, fc(Color32::from_gray(205), e * ky));

        // recursos
        let rows: [(&str, String, &str); 3] = [
            (
                "search",
                format!("{} itens documentados", ITEMS.len()),
                "Descrição, especificações e dica de bancada para cada placa e módulo.",
            ),
            (
                "star",
                "Favoritos e busca instantânea".to_string(),
                "Encontre qualquer item em segundos, mesmo sem acentos.",
            ),
            (
                "lock",
                "Login que fica salvo".to_string(),
                "Entre uma vez e continue logado neste computador.",
            ),
        ];
        for (i, (kind, title, sub)) in rows.iter().enumerate() {
            let a = easing::cubic_out(((t - 0.35 - i as f32 * 0.12) / 0.55).clamp(0.0, 1.0)) * e;
            let y = top + 214.0 + i as f32 * 74.0 + 12.0 * (1.0 - a);
            let ic = pos2(left + 22.0, y + 22.0);
            p.circle_filled(ic, 22.0, fc(PANEL, a));
            p.circle_stroke(ic, 22.0, st(1.0, fc(mix(LINE, ACCENT, 0.4), a)));
            let col = fc(ACCENT, a);
            match *kind {
                "search" => search_icon(p, ic, col),
                "star" => star_icon(p, ic, 9.0, false, fc(GOLD, a)),
                _ => lock_icon(p, ic, col),
            }
            p.text(pos2(left + 60.0, y + 12.0), Align2::LEFT_CENTER, title, sans(15.5), fc(TEXT, a));
            let sub = ellipsize(p, sub, sans(12.5), width - 60.0);
            p.text(pos2(left + 60.0, y + 34.0), Align2::LEFT_CENTER, sub, sans(12.5), fc(MUTED, a));
        }
    }

    fn drop_overlay(&self, ctx: &egui::Context, enabled: bool) {
        if enabled && ctx.input(|i| !i.raw.hovered_files.is_empty()) {
            let p = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("drop")));
            let r = ctx.screen_rect().shrink(14.0);
            p.rect_filled(r, 14.0, Color32::from_black_alpha(205));
            p.rect_stroke(r, 14.0, st(2.0, ACCENT));
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                "SOLTE A IMAGEM AQUI",
                bold(22.0),
                TEXT,
            );
            p.text(
                r.center() + vec2(0.0, 30.0),
                Align2::CENTER_CENTER,
                "ela vira sua foto de perfil",
                sans(13.0),
                MUTED,
            );
        }
    }

    // ───── Catálogo ─────
    fn content(&mut self, ui: &mut egui::Ui, time: f64) {
        let tt = time - self.login_t0;
        let width = ui.available_width();

        // cabeçalho: logo + título + estatísticas
        let (hr, _) = ui.allocate_exact_size(vec2(width, 64.0), Sense::hover());
        {
            let p = ui.painter().clone();
            let appear = easing::cubic_out((tt / 0.6).clamp(0.0, 1.0) as f32);
            if let Some(logo) = &self.logo {
                let lr = Rect::from_min_size(hr.left_top() + vec2(0.0, 6.0), vec2(52.0, 52.0));
                draw_logo(&p, logo, lr, appear);
            }
            let txt = typed_title(tt);
            let pos = pos2(hr.left() + 66.0, hr.top() + 24.0);
            if tt > 1.2 && (time % 7.0) < 0.15 {
                p.text(pos + vec2(3.0, -1.0), Align2::LEFT_CENTER, &txt, bold(30.0), alpha(ACCENT, 0.5));
                p.text(pos + vec2(-3.0, 1.0), Align2::LEFT_CENTER, &txt, bold(30.0), alpha(DANGER, 0.35));
            }
            p.text(pos, Align2::LEFT_CENTER, &txt, bold(30.0), TEXT);
            p.text(
                pos2(hr.left() + 66.0, hr.top() + 50.0),
                Align2::LEFT_CENTER,
                "Catálogo de hardware · cibersegurança · ESP32 · Arduino",
                sans(12.5),
                fc(MUTED, appear),
            );

            // estatísticas (somem em janelas estreitas, para não bater no perfil)
            if width > 980.0 {
                let stats = [
                    (ITEMS.len() as f32, "ITENS"),
                    (CATS.len() as f32, "CATEGORIAS"),
                    (self.favs.len() as f32, "FAVORITOS"),
                ];
                for (i, (v, lab)) in stats.iter().enumerate() {
                    let k = easing::cubic_out(((tt - 0.4 - i as f64 * 0.12) / 1.0).clamp(0.0, 1.0) as f32);
                    let shown = if *lab == "FAVORITOS" {
                        ui.ctx().animate_value_with_time(Id::new("stat_fav"), *v, 0.4)
                    } else {
                        v * k
                    };
                    let x = hr.right() - 290.0 - (2 - i) as f32 * 112.0;
                    let box_ = Rect::from_min_size(pos2(x, hr.top() + 4.0), vec2(100.0, 52.0));
                    p.rect_filled(box_, 10.0, fc(PANEL, k));
                    p.rect_stroke(box_, 10.0, st(1.0, fc(LINE, k)));
                    let colr = if *lab == "FAVORITOS" { GOLD } else { ACCENT };
                    p.text(
                        pos2(box_.left() + 14.0, box_.top() + 20.0),
                        Align2::LEFT_CENTER,
                        format!("{}", shown.round() as i32),
                        bold(21.0),
                        fc(colr, k),
                    );
                    p.text(
                        pos2(box_.left() + 14.0, box_.top() + 39.0),
                        Align2::LEFT_CENTER,
                        *lab,
                        mono(9.5),
                        fc(MUTED, k),
                    );
                }
            }
        }
        ui.add_space(8.0);
        let (lr, _) = ui.allocate_exact_size(vec2(width, 2.0), Sense::hover());
        {
            let k = easing::cubic_out(((tt - 0.2) / 1.4).clamp(0.0, 1.0) as f32);
            let seg = Rect::from_min_size(lr.min, vec2(lr.width() * k, 2.0));
            hgrad(ui.painter(), seg, alpha(ACCENT, 0.95), alpha(ACCENT, 0.0));
        }

        // busca + ordenar + contagem
        ui.add_space(16.0);
        let search_id = Id::new("search");
        let mut clear_q = false;
        let mut cycle_sort = false;
        // o texto de exemplo na busca vai sendo "digitado", trocando de sugestão
        const HINTS: [&str; 5] = [
            "Buscar por nome, chip ou uso…",
            "Experimente “esp32”",
            "Experimente “oled”",
            "Experimente “bateria”",
            "Experimente “nfc”",
        ];
        let cyc = (time / 3.2) as usize % HINTS.len();
        let local = (time % 3.2) as f32;
        let full = HINTS[cyc];
        let nchar = ((local * 22.0) as usize).min(full.chars().count());
        let hint_txt: String = full.chars().take(nchar).collect();
        let search_focused = ui.ctx().memory(|m| m.has_focus(search_id));
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            let (rect, _) = field(
                ui,
                search_id,
                &mut self.query,
                &hint_txt,
                false,
                420.0,
                42.0,
                1.0,
                38.0,
                38.0,
            );
            search_icon(
                ui.painter(),
                pos2(rect.left() + 20.0, rect.center().y),
                if ui.ctx().memory(|m| m.has_focus(search_id)) { ACCENT } else { MUTED },
            );
            if !self.query.is_empty() {
                let xr = Rect::from_center_size(pos2(rect.right() - 20.0, rect.center().y), vec2(26.0, 26.0));
                let r = ui.interact(xr, search_id.with("x"), Sense::click());
                if r.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                close_icon(
                    ui.painter(),
                    xr.center(),
                    4.5,
                    if r.hovered() { TEXT } else { MUTED },
                );
                if r.clicked() {
                    clear_q = true;
                }
            } else if !search_focused {
                // "tecla" / indicando o atalho
                let kr = Rect::from_center_size(pos2(rect.right() - 24.0, rect.center().y), vec2(22.0, 22.0));
                ui.painter().rect_filled(kr, 6.0, PANEL2);
                ui.painter().rect_stroke(kr, 6.0, st(1.0, LINE));
                ui.painter().text(kr.center(), Align2::CENTER_CENTER, "/", mono(12.5), MUTED);
            }
            let gap = rect.right() + 12.0 - ui.cursor().left();
            if gap > 0.0 {
                ui.add_space(gap);
            }
            if ghost_button(ui, self.sort.label(), 150.0, self.sort != Sort::Default, ACCENT, None, 1.0) {
                cycle_sort = true;
            }
        });
        if clear_q {
            self.query.clear();
        }
        if cycle_sort {
            self.sort = self.sort.next();
        }

        // filtros
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            if chip(ui, "TODOS", Some(ITEMS.len()), self.filter == Filter::All, TEXT, false) {
                self.filter = Filter::All;
            }
            if chip(ui, "FAVORITOS", Some(self.favs.len()), self.filter == Filter::Favs, GOLD, true) {
                self.filter = Filter::Favs;
            }
            for c in CATS {
                let n = ITEMS.iter().filter(|i| i.cat == c).count();
                if chip(ui, c.plural(), Some(n), self.filter == Filter::Cat(c), c.color(), true) {
                    self.filter = Filter::Cat(c);
                }
            }
        });

        // lista filtrada
        let key = (self.filter, self.sort, self.query.clone());
        if key != self.last_key {
            self.grid_t0 = time;
            self.last_key = key;
        }
        let q = norm(self.query.trim());
        let mut list: Vec<usize> = (0..ITEMS.len())
            .filter(|&i| {
                let it = &ITEMS[i];
                let ok_f = match self.filter {
                    Filter::All => true,
                    Filter::Favs => self.favs.iter().any(|f| f == it.name),
                    Filter::Cat(c) => it.cat == c,
                };
                ok_f && q.split_whitespace().all(|w| self.blobs[i].contains(w))
            })
            .collect();
        match self.sort {
            Sort::Default => {}
            Sort::Az => list.sort_by_key(|&i| norm(ITEMS[i].name)),
            Sort::Level => list.sort_by_key(|&i| ITEMS[i].level.bars()),
        }
        self.visible = list.clone();

        ui.add_space(10.0);
        let title = match self.filter {
            Filter::All => "Todos os itens".to_string(),
            Filter::Favs => "Seus favoritos".to_string(),
            Filter::Cat(c) => c.plural().to_string(),
        };
        let shown = ui
            .ctx()
            .animate_value_with_time(Id::new("count_anim"), list.len() as f32, 0.35)
            .round() as usize;
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).font(sans(15.0)).color(TEXT));
            ui.label(
                RichText::new(format!(
                    "· {} {}",
                    shown,
                    if shown == 1 { "resultado" } else { "resultados" }
                ))
                .font(mono(11.5))
                .color(DIM),
            );
        });
        ui.add_space(8.0);

        let t0 = self.grid_t0;
        let active_idx = if self.open {
            self.sel.as_ref().map(|s| s.idx)
        } else {
            None
        };
        let scroll_to = self.scroll_to.take();
        let imgs = &self.imgs;
        let favs = &self.favs;
        let mut clicked: Option<usize> = None;
        let mut fav_click: Option<usize> = None;
        let mut laid: Vec<(usize, Rect)> = Vec::new();
        let filter = self.filter;
        let mut clear_filters = false;
        let avail_w = ui.available_width();
        let mut area = egui::ScrollArea::vertical().auto_shrink([false, false]);
        if std::mem::take(&mut self.scroll_top) {
            area = area.vertical_scroll_offset(0.0);
        }
        let scroll_out = area.show(ui, |ui| {
                ui.add_space(4.0);
                if list.is_empty() {
                    ui.add_space(50.0);
                    ui.vertical_centered(|ui| {
                        let (msg, sub) = if filter == Filter::Favs && q.is_empty() {
                            (
                                "Você ainda não tem favoritos",
                                "Passe o mouse sobre um item e clique na estrela para guardá-lo aqui.",
                            )
                        } else {
                            (
                                "Nenhum item encontrado",
                                "Tente outro termo, como “esp32”, “oled” ou “bateria”.",
                            )
                        };
                        ui.label(RichText::new(msg).font(sans(20.0)).color(TEXT));
                        ui.add_space(4.0);
                        ui.label(RichText::new(sub).font(sans(13.0)).color(MUTED));
                        ui.add_space(16.0);
                        if ghost_button(ui, "LIMPAR FILTROS", 160.0, false, ACCENT, None, 1.0) {
                            clear_filters = true;
                        }
                    });
                }
                let cols = (((avail_w + GAP) / (CARD_W + GAP)).floor() as usize).max(1);
                let used = cols as f32 * (CARD_W + GAP) - GAP;
                let pad = ((avail_w - used) / 2.0).max(0.0);
                ui.spacing_mut().item_spacing = vec2(GAP, GAP);
                for (row, chunk) in list.chunks(cols).enumerate() {
                    ui.horizontal(|ui| {
                        ui.add_space(pad);
                        ui.spacing_mut().item_spacing.x = GAP;
                        for (ci, &idx) in chunk.iter().enumerate() {
                            let order = row * cols + ci;
                            let active = active_idx == Some(idx);
                            let (c, f) = card(
                                ui,
                                &ITEMS[idx],
                                idx,
                                order,
                                t0,
                                active,
                                imgs[idx].as_ref(),
                                favs.iter().any(|n| n == ITEMS[idx].name),
                                &mut laid,
                            );
                            if scroll_to == Some(idx) {
                                if let Some((_, r)) = laid.last() {
                                    ui.scroll_to_rect(r.expand2(vec2(0.0, 30.0)), Some(Align::Center));
                                }
                            }
                            if c {
                                clicked = Some(idx);
                            }
                            if f {
                                fav_click = Some(idx);
                            }
                        }
                    });
                }
                ui.add_space(28.0);
            });
        self.scroll_y = scroll_out.state.offset.y;
        for (i, r) in laid {
            self.rects[i] = Some(r);
        }
        if clear_filters {
            self.query.clear();
            self.filter = Filter::All;
        }
        if let Some(idx) = fav_click {
            self.toggle_fav(idx, time);
        }
        if let Some(idx) = clicked {
            self.sel = Some(Sel { idx, t0: time });
            self.open = true;
            self.menu_open = false;
        }
    }

    // ───── Botão flutuante "voltar ao topo" ─────
    fn scroll_fab(&mut self, ctx: &egui::Context) {
        let show = ctx.animate_bool_with_time(Id::new("fab_t"), self.scroll_y > 360.0 && !self.open, 0.25);
        if show <= 0.01 {
            return;
        }
        let mut go = false;
        egui::Area::new(Id::new("fab"))
            .order(Order::Middle)
            .anchor(Align2::RIGHT_BOTTOM, vec2(-34.0, -58.0 + 12.0 * (1.0 - show)))
            .show(ctx, |ui| {
                let (rect, resp) = ui.allocate_exact_size(vec2(46.0, 46.0), Sense::click());
                let hv = ctx.animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.15);
                if resp.hovered() {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let p = ui.painter();
                let c = rect.center() + vec2(0.0, -2.0 * hv);
                glow(p, c, 40.0, ACCENT, 0.16 * hv * show);
                p.circle_filled(c, 22.0, fc(mix(PANEL2, Color32::from_rgb(14, 40, 48), hv), show));
                p.circle_stroke(c, 22.0, st(1.3, fc(mix(LINE, ACCENT, 0.3 + 0.7 * hv), show)));
                let col = fc(mix(MUTED, ACCENT, hv), show);
                p.line_segment([c + vec2(-6.0, 3.0), c + vec2(0.0, -3.5)], st(2.0, col));
                p.line_segment([c + vec2(0.0, -3.5), c + vec2(6.0, 3.0)], st(2.0, col));
                if resp.clicked() {
                    go = true;
                }
            });
        if go {
            self.scroll_top = true;
        }
    }

    // ───── Rodapé ─────
    fn footer(&mut self, ctx: &egui::Context, time: f64) {
        egui::TopBottomPanel::bottom("footer")
            .exact_height(34.0)
            .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(32.0, 0.0)))
            .show(ctx, |ui| {
                let r = ui.max_rect();
                let p = ui.painter();
                p.line_segment(
                    [r.left_top() - vec2(32.0, 0.0), r.right_top() + vec2(32.0, 0.0)],
                    st(1.0, LINE),
                );
                let pulse = 0.5 + 0.5 * (time as f32 * 2.4).sin();
                let dc = pos2(r.left() + 6.0, r.center().y);
                glow(p, dc, 9.0, OK, 0.5 * pulse);
                p.circle_filled(dc, 3.0, OK);
                let saved = match self.store.session_days_left() {
                    Some(d) => format!("SALVA POR {d} DIAS"),
                    None => "SÓ ESTA VEZ".to_string(),
                };
                p.text(
                    pos2(r.left() + 20.0, r.center().y),
                    Align2::LEFT_CENTER,
                    format!(
                        "SESSÃO ATIVA · {} · {}",
                        self.user.clone().unwrap_or_default().to_uppercase(),
                        saved
                    ),
                    mono(10.5),
                    MUTED,
                );
                let hints = if r.width() > 900.0 {
                    format!("/ buscar    ← → navegar    F favoritar    ESC fechar    v{VERSION}")
                } else {
                    format!("v{VERSION}")
                };
                p.text(
                    pos2(r.right(), r.center().y),
                    Align2::RIGHT_CENTER,
                    hints,
                    mono(10.5),
                    DIM,
                );
            });
    }

    // ───── Toasts ─────
    fn draw_toasts(&mut self, ctx: &egui::Context, time: f64) {
        self.toasts.retain(|t| time - t.t0 < 3.0);
        let screen = ctx.screen_rect();
        for (i, t) in self.toasts.iter().enumerate() {
            let age = (time - t.t0) as f32;
            if age < 0.0 {
                continue;
            }
            let inn = easing::cubic_out((age / 0.3).clamp(0.0, 1.0));
            let out = ((age - 2.6) / 0.4).clamp(0.0, 1.0);
            let a = inn * (1.0 - out);
            let font = sans(13.0);
            let w = ctx
                .fonts(|f| f.layout_no_wrap(t.msg.clone(), font.clone(), Color32::WHITE).size().x)
                + 52.0;
            let h = 40.0;
            let y = screen.bottom() - 62.0 - (self.toasts.len() - 1 - i) as f32 * 48.0 + 14.0 * (1.0 - inn);
            let r = Rect::from_center_size(pos2(screen.center().x, y), vec2(w, h));
            let p = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::new(("toast", i))));
            p.rect_filled(r.translate(vec2(0.0, 3.0)), 12.0, Color32::from_black_alpha((90.0 * a) as u8));
            p.rect_filled(r, 12.0, fc(PANEL2, a));
            p.rect_stroke(r, 12.0, st(1.0, fc(mix(LINE, t.color, 0.6), a)));
            p.circle_filled(pos2(r.left() + 20.0, r.center().y - 1.0), 4.0, fc(t.color, a));
            glow(&p, pos2(r.left() + 20.0, r.center().y - 1.0), 12.0, t.color, 0.35 * a);
            p.text(
                pos2(r.left() + 34.0, r.center().y - 1.0),
                Align2::LEFT_CENTER,
                &t.msg,
                font,
                fc(TEXT, a),
            );
            // barrinha que mostra o tempo restante do aviso
            let prog = (1.0 - age / 3.0).clamp(0.0, 1.0);
            p.rect_filled(
                Rect::from_min_size(pos2(r.left() + 14.0, r.bottom() - 4.0), vec2((w - 28.0) * prog, 2.0)),
                1.0,
                alpha(t.color, 0.55 * a),
            );
        }
    }

    // ───── Perfil (canto superior direito) ─────
    fn profile_chip(&mut self, ctx: &egui::Context, time: f64) {
        let Some(name) = self.user.clone() else { return };
        let mut toggle = false;
        let mut change_photo = false;
        let mut logout = false;
        let letter = name.chars().next().unwrap_or('?').to_uppercase().to_string();
        let avatar = self.avatar.clone();
        let e = easing::cubic_out(((time - self.login_t0) / 0.8).clamp(0.0, 1.0) as f32);
        let menu_open = self.menu_open;
        let session_line = match self.store.session_days_left() {
            Some(1) => "Sessão salva · expira em 1 dia".to_string(),
            Some(d) => format!("Sessão salva · expira em {d} dias"),
            None => "Sessão temporária · não fica salva".to_string(),
        };

        egui::Area::new(Id::new("profile_chip"))
            .order(Order::Middle)
            .anchor(Align2::RIGHT_TOP, vec2(-32.0, 24.0))
            .show(ctx, |ui| {
                let w = 78.0 + name.chars().count() as f32 * 8.6;
                let (rect, resp) = ui.allocate_exact_size(vec2(w, 48.0), Sense::click());
                let hv = ctx.animate_bool_with_time(resp.id.with("h"), resp.hovered() || menu_open, 0.15);
                if resp.hovered() {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let p = ui.painter();
                p.rect_filled(rect, 24.0, fc(mix(PANEL, PANEL2, hv), e));
                p.rect_stroke(rect, 24.0, st(1.0, fc(mix(LINE, ACCENT, hv), e)));
                draw_avatar(p, pos2(rect.left() + 25.0, rect.center().y), 18.0, avatar.as_ref(), &letter, e);
                p.text(
                    pos2(rect.left() + 52.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    &name,
                    mono(13.5),
                    fc(TEXT, e),
                );
                // setinha do menu
                let ac = pos2(rect.right() - 18.0, rect.center().y);
                let d = if menu_open { -1.0 } else { 1.0 };
                p.line_segment([ac + vec2(-4.0, -1.5 * d), ac + vec2(0.0, 2.5 * d)], st(1.6, fc(MUTED, e)));
                p.line_segment([ac + vec2(0.0, 2.5 * d), ac + vec2(4.0, -1.5 * d)], st(1.6, fc(MUTED, e)));
                if resp.clicked() {
                    toggle = true;
                }
            });

        let mt = ctx.animate_bool_with_time(Id::new("menu_t"), self.menu_open, 0.18);
        if mt > 0.0 {
            egui::Area::new(Id::new("profile_menu"))
                .order(Order::Middle)
                .anchor(Align2::RIGHT_TOP, vec2(-32.0, 80.0 - 8.0 * (1.0 - mt)))
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(fc(PANEL, mt))
                        .stroke(st(1.0, fc(LINE, mt)))
                        .rounding(12.0)
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.set_width(210.0);
                            let (hr, _) = ui.allocate_exact_size(vec2(210.0, 40.0), Sense::hover());
                            let p = ui.painter();
                            p.text(
                                hr.left_top() + vec2(8.0, 12.0),
                                Align2::LEFT_CENTER,
                                format!("@{}", name),
                                mono(12.5),
                                fc(TEXT, mt),
                            );
                            p.text(
                                hr.left_top() + vec2(8.0, 29.0),
                                Align2::LEFT_CENTER,
                                &session_line,
                                sans(11.0),
                                fc(DIM, mt),
                            );
                            p.line_segment(
                                [hr.left_bottom() + vec2(0.0, 1.0), hr.right_bottom() + vec2(0.0, 1.0)],
                                st(1.0, fc(LINE, mt)),
                            );
                            ui.add_space(4.0);
                            if menu_item(ui, "TROCAR FOTO", 210.0, false) {
                                change_photo = true;
                            }
                            if menu_item(ui, "SAIR DA CONTA", 210.0, true) {
                                logout = true;
                            }
                        });
                });
        }

        if toggle {
            self.menu_open = !self.menu_open;
        }
        if change_photo {
            self.menu_open = false;
            self.pick_file(ctx, time);
        }
        if logout {
            self.logout(time);
        }
    }

    fn navigate(&mut self, dir: i32, time: f64) {
        let Some(sel) = &self.sel else { return };
        let Some(pos) = self.visible.iter().position(|&i| i == sel.idx) else {
            return;
        };
        let n = self.visible.len() as i32;
        if n < 2 {
            return;
        }
        let np = (pos as i32 + dir).rem_euclid(n) as usize;
        let idx = self.visible[np];
        self.sel = Some(Sel { idx, t0: time });
        self.scroll_to = Some(idx);
    }

    // ───── Botão destacado + painel de detalhes ao lado + fundo fosco ─────
    fn spotlight(&mut self, ctx: &egui::Context, time: f64) {
        let t = ctx.animate_bool_with_time(Id::new("panel_t"), self.open, 0.35);
        let Some(sel) = &self.sel else { return };
        if t <= 0.0 {
            return;
        }
        let (idx, t0) = (sel.idx, sel.t0);
        let Some(card_rect) = self.rects.get(idx).copied().flatten() else {
            return;
        };
        let it = &ITEMS[idx];
        let col = it.cat.color();
        let fav = self.is_fav(it.name);
        let e = easing::cubic_out(t);
        let screen = ctx.screen_rect();
        let mut close = false;
        let mut nav = 0;
        let mut toggle_fav = false;
        let mut copy = false;
        let pos_in_list = self.visible.iter().position(|&i| i == idx);
        let total = self.visible.len();
        // itens da mesma categoria que também estão na lista atual (para pular entre eles)
        let mut rel: Vec<usize> = (0..ITEMS.len())
            .filter(|&i| i != idx && ITEMS[i].cat == it.cat && self.visible.contains(&i))
            .collect();
        rel.sort_by_key(|&i| (i as i32 - idx as i32).abs());
        rel.truncate(3);
        let mut jump: Option<usize> = None;
        let imgs = &self.imgs;

        // 1) fundo escurecido e "fosco" (clicar fora fecha)
        egui::Area::new(Id::new("dim"))
            .order(Order::Foreground)
            .fixed_pos(screen.min)
            .constrain(false)
            .show(ctx, |ui| {
                let (r, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
                let p = ui.painter();
                p.rect_filled(r, 0.0, Color32::from_black_alpha((215.0 * e) as u8));
                glow(p, card_rect.center(), 380.0, col, 0.16 * e);
                let line = Color32::from_white_alpha((7.0 * e) as u8);
                let mut y = r.top();
                while y < r.bottom() {
                    p.line_segment([pos2(r.left(), y), pos2(r.right(), y)], st(1.0, line));
                    y += 4.0;
                }
                if resp.clicked() {
                    close = true;
                }
            });

        // 2) o card selecionado, em destaque, com pulso
        let spot = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::new("spot_card")));
        let scaled = card_rect.scale_from_center(1.0 + 0.05 * e);
        for k in 0..3 {
            let ph = (((time - t0) * 0.9) as f32 + k as f32 / 3.0) % 1.0;
            spot.rect_stroke(
                scaled.expand(4.0 + ph * 26.0),
                16.0,
                st(1.2, alpha(col, 0.75 * (1.0 - ph) * e)),
            );
        }
        paint_card(
            &spot,
            scaled,
            it,
            time,
            1.0,
            self.imgs[idx].as_ref(),
            fav,
            CardFx { hov: 1.0, ..Default::default() },
            None,
        );

        // 3) painel ao lado do card (direita; se faltar espaço, esquerda)
        let pw = 400.0_f32;
        let ph = ctx
            .data(|d| d.get_temp::<f32>(Id::new("detail_h")))
            .unwrap_or(690.0)
            .min(screen.height() - 32.0);
        let right = scaled.right() + 24.0 + pw <= screen.right() - 16.0;
        let base_x = if right {
            scaled.right() + 24.0
        } else {
            scaled.left() - 24.0 - pw
        };
        let dir = if right { 1.0 } else { -1.0 };
        let x = base_x - dir * 48.0 * (1.0 - e);
        let y = scaled
            .top()
            .clamp(16.0, (screen.bottom() - ph - 16.0).max(16.0));

        // conector animado entre o card e o painel
        let cy = scaled.center().y.clamp(y + 24.0, y + ph - 24.0);
        let (ax, bx) = if right {
            (scaled.right(), base_x)
        } else {
            (scaled.left(), base_x + pw)
        };
        let bx_now = ax + (bx - ax) * e;
        spot.line_segment([pos2(ax, cy), pos2(bx_now, cy)], st(1.5, alpha(col, e)));
        let dot = ax + (bx_now - ax) * (((time * 0.8) % 1.0) as f32);
        glow(&spot, pos2(dot, cy), 10.0, col, 0.7 * e);
        spot.circle_filled(pos2(dot, cy), 3.0, fc(TEXT, e));

        // staggered fade-in do conteúdo
        let cc = |delay: f64| -> f32 {
            easing::cubic_out(((time - t0 - delay) / 0.45).clamp(0.0, 1.0) as f32) * e
        };

        let detail = egui::Area::new(Id::new("detail"))
            .order(Order::Tooltip)
            .fixed_pos(pos2(x, y))
            .constrain(false)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(fc(PANEL, e))
                    .stroke(st(1.0, fc(mix(LINE, col, 0.55), e)))
                    .rounding(16.0)
                    .inner_margin(egui::Margin::same(22.0))
                    .show(ui, |ui| {
                        let iw = pw - 44.0;
                        ui.set_width(iw);
                        ui.set_min_height(520.0);

                        // barra superior: rótulo, posição, ◀ ▶ e fechar
                        let (bar, _) = ui.allocate_exact_size(vec2(iw, 28.0), Sense::hover());
                        let a0 = cc(0.0);
                        ui.painter().text(
                            bar.left_center(),
                            Align2::LEFT_CENTER,
                            "// DETALHES",
                            mono(11.5),
                            fc(DIM, a0),
                        );
                        let mut bx = bar.right() - 14.0;
                        let xr = Rect::from_center_size(pos2(bx, bar.center().y), vec2(28.0, 28.0));
                        let r = ui.interact(xr, Id::new("close_x"), Sense::click());
                        let hv = ctx.animate_bool_with_time(Id::new("close_h"), r.hovered(), 0.12);
                        if r.hovered() {
                            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        close_icon(ui.painter(), xr.center(), 6.0 + 1.5 * hv, fc(mix(MUTED, TEXT, hv), a0));
                        if r.clicked() {
                            close = true;
                        }
                        bx -= 34.0;
                        for (d, id) in [(1.0_f32, "nav_next"), (-1.0_f32, "nav_prev")] {
                            let nr = Rect::from_center_size(pos2(bx, bar.center().y), vec2(28.0, 28.0));
                            let r = ui.interact(nr, Id::new(id), Sense::click());
                            let hv = ctx.animate_bool_with_time(Id::new((id, "h")), r.hovered(), 0.12);
                            if r.hovered() {
                                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            ui.painter().circle_filled(nr.center(), 13.0, alpha(col, 0.18 * hv * a0));
                            arrow_icon(ui.painter(), nr.center(), d, fc(mix(MUTED, TEXT, hv), a0));
                            if r.clicked() {
                                nav = d as i32;
                            }
                            bx -= 30.0;
                        }
                        if let Some(pos) = pos_in_list {
                            ui.painter().text(
                                pos2(bx + 6.0, bar.center().y),
                                Align2::RIGHT_CENTER,
                                format!("{:02}/{:02}", pos + 1, total),
                                mono(11.5),
                                fc(MUTED, a0),
                            );
                        }

                        ui.add_space(8.0);
                        ui.label(RichText::new(it.name).font(sans(25.0)).color(fc(TEXT, cc(0.05))));
                        ui.add_space(4.0);
                        let (ln, _) = ui.allocate_exact_size(vec2(iw, 3.0), Sense::hover());
                        let lk = easing::cubic_out(((time - t0) / 0.7).clamp(0.0, 1.0) as f32);
                        hgrad(
                            ui.painter(),
                            Rect::from_min_size(ln.min, vec2(ln.width() * lk * e, 3.0)),
                            fc(col, e),
                            alpha(col, 0.0),
                        );

                        // selos: categoria e nível
                        ui.add_space(12.0);
                        let a1 = cc(0.1);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            // categoria
                            let lab = it.cat.label();
                            let lw = ui
                                .painter()
                                .layout_no_wrap(lab.to_string(), mono(11.0), Color32::WHITE)
                                .size()
                                .x;
                            let (r, _) = ui.allocate_exact_size(vec2(lw + 30.0, 26.0), Sense::hover());
                            let p = ui.painter();
                            p.rect_filled(r, 13.0, alpha(col, 0.16 * a1));
                            p.rect_stroke(r, 13.0, st(1.0, alpha(col, 0.55 * a1)));
                            p.circle_filled(pos2(r.left() + 13.0, r.center().y), 3.2, fc(col, a1));
                            p.text(pos2(r.left() + 23.0, r.center().y), Align2::LEFT_CENTER, lab, mono(11.0), fc(TEXT, a1));
                            // nível
                            let lv = it.level.label();
                            let lw = ui
                                .painter()
                                .layout_no_wrap(lv.to_string(), mono(11.0), Color32::WHITE)
                                .size()
                                .x;
                            let (r, _) = ui.allocate_exact_size(vec2(lw + 54.0, 26.0), Sense::hover());
                            let p = ui.painter();
                            p.rect_stroke(r, 13.0, st(1.0, fc(LINE, a1)));
                            level_bars(p, pos2(r.left() + 12.0, r.center().y), it.level, col, a1);
                            p.text(pos2(r.left() + 40.0, r.center().y), Align2::LEFT_CENTER, lv, mono(11.0), fc(MUTED, a1));
                        });

                        // especificações como etiquetas
                        ui.add_space(12.0);
                        let a2 = cc(0.18);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                            for spec in it.chip.split(" · ") {
                                let sw = ui
                                    .painter()
                                    .layout_no_wrap(spec.to_string(), mono(11.0), Color32::WHITE)
                                    .size()
                                    .x;
                                let (r, _) = ui.allocate_exact_size(vec2(sw + 20.0, 24.0), Sense::hover());
                                let p = ui.painter();
                                p.rect_filled(r, 7.0, fc(PANEL2, a2));
                                p.rect_stroke(r, 7.0, st(1.0, fc(LINE, a2)));
                                p.text(r.center(), Align2::CENTER_CENTER, spec, mono(11.0), fc(Color32::from_gray(190), a2));
                            }
                        });

                        ui.add_space(12.0);
                        ui.label(
                            RichText::new(it.desc)
                                .font(sans(13.5))
                                .color(fc(Color32::from_gray(208), cc(0.26))),
                        );

                        // dica de bancada em destaque
                        ui.add_space(12.0);
                        let at = cc(0.32);
                        let tip_box = egui::Frame::none()
                            .fill(alpha(col, 0.08 * at))
                            .stroke(st(1.0, alpha(col, 0.28 * at)))
                            .rounding(10.0)
                            .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                            .show(ui, |ui| {
                                ui.set_width(iw - 28.0);
                                ui.label(
                                    RichText::new("// DICA DE BANCADA")
                                        .font(mono(10.5))
                                        .color(fc(col, at)),
                                );
                                ui.add_space(3.0);
                                ui.label(
                                    RichText::new(it.tip)
                                        .font(sans(12.5))
                                        .color(fc(Color32::from_gray(205), at)),
                                );
                            });
                        let tr = tip_box.response.rect;
                        ui.painter().rect_filled(
                            Rect::from_min_size(tr.min + vec2(0.0, 9.0), vec2(2.5, tr.height() - 18.0)),
                            1.0,
                            fc(col, at),
                        );

                        ui.add_space(12.0);
                        ui.label(
                            RichText::new("// USOS COMUNS")
                                .font(mono(11.0))
                                .color(fc(DIM, cc(0.38))),
                        );
                        ui.add_space(3.0);
                        for (i, u) in it.uses.iter().enumerate() {
                            let a = cc(0.42 + 0.07 * i as f64);
                            let (r, _) = ui.allocate_exact_size(vec2(iw, 20.0), Sense::hover());
                            let p = ui.painter();
                            let c = pos2(r.left() + 6.0 - 6.0 * (1.0 - a), r.center().y);
                            p.line_segment([c + vec2(-2.0, -4.0), c + vec2(2.5, 0.0)], st(1.6, fc(col, a)));
                            p.line_segment([c + vec2(2.5, 0.0), c + vec2(-2.0, 4.0)], st(1.6, fc(col, a)));
                            p.text(
                                pos2(r.left() + 20.0, r.center().y),
                                Align2::LEFT_CENTER,
                                *u,
                                sans(13.5),
                                fc(Color32::from_gray(214), a),
                            );
                        }

                        // itens relacionados (mesma categoria): clique para pular
                        if !rel.is_empty() {
                            ui.add_space(12.0);
                            let ar = cc(0.6);
                            ui.label(
                                RichText::new("// RELACIONADOS")
                                    .font(mono(11.0))
                                    .color(fc(DIM, ar)),
                            );
                            ui.add_space(5.0);
                            let gapx = 8.0;
                            let cw = (iw - 2.0 * gapx) / 3.0;
                            let (row, _) = ui.allocate_exact_size(vec2(iw, 46.0), Sense::hover());
                            for (k, &ri) in rel.iter().enumerate() {
                                let rr = Rect::from_min_size(
                                    row.min + vec2(k as f32 * (cw + gapx), 0.0),
                                    vec2(cw, 46.0),
                                );
                                let resp = ui.interact(rr, Id::new(("rel", k)), Sense::click());
                                let hv = ctx.animate_bool_with_time(Id::new(("rel_h", k)), resp.hovered(), 0.15);
                                if resp.hovered() {
                                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                let rc = ITEMS[ri].cat.color();
                                let pnt = ui.painter();
                                pnt.rect_filled(rr, 9.0, fc(mix(PANEL2, Color32::from_rgb(22, 30, 40), hv), ar));
                                pnt.rect_stroke(rr, 9.0, st(1.0, fc(mix(LINE, rc, hv), ar)));
                                let th = Rect::from_min_size(rr.min + vec2(6.0, 6.0), vec2(34.0, 34.0));
                                match imgs[ri].as_ref() {
                                    Some(tex) => {
                                        pnt.rect_filled(th, 6.0, gray(244, ar));
                                        let sz = tex.size_vec2();
                                        let kk = ((th.width() - 4.0) / sz.x).min((th.height() - 4.0) / sz.y);
                                        pnt.with_clip_rect(th).image(
                                            tex.id(),
                                            Rect::from_center_size(th.center(), sz * kk),
                                            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                            gray(255, ar),
                                        );
                                    }
                                    None => {
                                        pnt.rect_filled(th, 6.0, fc(Color32::from_rgb(6, 9, 13), ar));
                                        pnt.circle_filled(th.center(), 4.0, fc(rc, ar));
                                    }
                                }
                                let mut job = egui::text::LayoutJob::single_section(
                                    ITEMS[ri].name.to_string(),
                                    egui::TextFormat::simple(sans(11.5), fc(mix(MUTED, TEXT, hv), ar)),
                                );
                                job.wrap = egui::text::TextWrapping {
                                    max_width: cw - 52.0,
                                    max_rows: 2,
                                    break_anywhere: false,
                                    overflow_character: Some('…'),
                                };
                                let galley = pnt.layout_job(job);
                                let gp = pos2(rr.left() + 46.0, rr.center().y - galley.size().y / 2.0);
                                pnt.galley(gp, galley, Color32::WHITE);
                                if resp.clicked() {
                                    jump = Some(ri);
                                }
                            }
                        }

                        ui.add_space(14.0);
                        let ab = cc(0.7);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 10.0;
                            let label = if fav { "FAVORITADO" } else { "FAVORITAR" };
                            if ghost_button(ui, label, 172.0, fav, GOLD, Some(fav), ab) {
                                toggle_fav = true;
                            }
                            if ghost_button(ui, "COPIAR FICHA", 160.0, false, col, None, ab) {
                                copy = true;
                            }
                        });
                    });
            });
        ctx.data_mut(|d| d.insert_temp(Id::new("detail_h"), detail.response.rect.height()));

        if close {
            self.open = false;
        }
        if let Some(j) = jump {
            self.sel = Some(Sel { idx: j, t0: time });
            self.scroll_to = Some(j);
        }
        if nav != 0 {
            self.navigate(nav, time);
        }
        if toggle_fav {
            self.toggle_fav(idx, time);
        }
        if copy {
            let text = format!(
                "{}\n{}\n\n{}\n\nUsos comuns:\n{}",
                it.name,
                it.chip,
                it.desc,
                it.uses
                    .iter()
                    .map(|u| format!("- {u}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            ctx.copy_text(text);
            self.toast("Ficha copiada para a área de transferência", OK, time);
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context, time: f64) {
        let typing = ctx.wants_keyboard_input();
        // "/" ou Ctrl+F focam a busca
        let slash = ctx.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::Text(t) if t == "/"))
        });
        let ctrl_f = ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::F));
        if (slash && !typing) || ctrl_f {
            ctx.memory_mut(|m| m.request_focus(Id::new("search")));
            self.open = false;
            self.menu_open = false;
            if slash {
                ctx.input_mut(|i| {
                    i.events
                        .retain(|e| !matches!(e, egui::Event::Text(t) if t == "/"));
                });
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.menu_open {
                self.menu_open = false;
            } else if self.open {
                self.open = false;
            } else if typing || !self.query.is_empty() {
                self.query.clear();
                ctx.memory_mut(|m| m.surrender_focus(Id::new("search")));
            }
        }
        if self.open && !typing {
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                self.navigate(1, time);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                self.navigate(-1, time);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::F)) {
                if let Some(idx) = self.sel.as_ref().map(|s| s.idx) {
                    self.toggle_fav(idx, time);
                }
            }
        }
    }
}

impl eframe::App for CyberDesk {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let time = ctx.input(|i| i.time);
        ctx.request_repaint_after(Duration::from_millis(16));
        if self.boot_t0 == 0.0 {
            self.boot_t0 = time;
        }
        draw_background(ctx, time);

        // imagem de perfil por arrastar e soltar
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        for f in dropped {
            let bytes = if let Some(b) = &f.bytes {
                Some(b.to_vec())
            } else if let Some(p) = &f.path {
                std::fs::read(p).ok()
            } else {
                None
            };
            if let Some(b) = bytes {
                self.apply_avatar(ctx, b, time);
                break;
            }
        }

        match self.screen {
            Screen::Splash => {
                self.splash(ctx, time);
                return;
            }
            Screen::Login => {
                if self.pending.is_some() {
                    self.success_overlay(ctx, time);
                } else {
                    self.login_screen(ctx, time);
                }
                self.draw_toasts(ctx, time);
                return;
            }
            Screen::Catalog => {}
        }

        self.handle_keys(ctx, time);
        self.footer(ctx, time);
        self.scroll_fab(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(32.0, 22.0)))
            .show(ctx, |ui| {
                self.content(ui, time);
            });

        self.profile_chip(ctx, time);
        self.spotlight(ctx, time);
        self.draw_fx(ctx, time);
        self.drop_overlay(ctx, true);
        self.draw_toasts(ctx, time);
    }
}

fn app_icon() -> egui::IconData {
    match image::load_from_memory(LOGO_PNG) {
        Ok(img) => {
            let img = img.thumbnail(256, 256).to_rgba8();
            egui::IconData {
                width: img.width(),
                height: img.height(),
                rgba: img.into_raw(),
            }
        }
        Err(_) => egui::IconData::default(),
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 720.0])
            .with_title("CyberDesk")
            .with_icon(app_icon())
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "CyberDesk",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            setup_style(&cc.egui_ctx);
            Ok(Box::new(CyberDesk::new(&cc.egui_ctx)))
        }),
    )
}
