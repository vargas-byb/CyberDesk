#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auth;

use eframe::egui::emath::easing;
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, ColorImage, FontId, Id, LayerId, Order, Painter, Pos2, Rect,
    RichText, Sense, Stroke, TextureHandle, TextureOptions,
};
use include_dir::{include_dir, Dir};
use std::f32::consts::PI;
use std::time::Duration;

static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/items");

const CARD_W: f32 = 224.0;
const CARD_H: f32 = 226.0;
const ICON_H: f32 = 148.0;

// ───────────────────────────── Dados ─────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Cat {
    Esp32,
    Arduino,
    Display,
    Modulo,
    Energia,
    Pcb,
    Comp,
}

impl Cat {
    fn label(self) -> &'static str {
        match self {
            Cat::Esp32 => "ESP32",
            Cat::Arduino => "ARDUINO",
            Cat::Display => "DISPLAY",
            Cat::Modulo => "MÓDULO",
            Cat::Energia => "ENERGIA",
            Cat::Pcb => "PCB",
            Cat::Comp => "COMPONENTES",
        }
    }
}

#[derive(Clone, Copy)]
enum Icon {
    Board {
        aspect: f32,
        pins: u8,
        usb: bool,
        ant: bool,
        chips: u8,
    },
    Oled,
    Lcd {
        rows: u8,
    },
    Nfc,
    Cyd,
    Antenna,
    Battery {
        mah: u32,
    },
    Pcb,
    Perf,
}

const fn board(aspect: f32, pins: u8, usb: bool, ant: bool, chips: u8) -> Icon {
    Icon::Board {
        aspect,
        pins,
        usb,
        ant,
        chips,
    }
}

struct Item {
    name: &'static str,
    cat: Cat,
    icon: Icon,
    chip: &'static str,
    desc: &'static str,
}

const ITEMS: &[Item] = &[
    // ── ESP32 ──
    Item {
        name: "ESP32 DevKit V1",
        cat: Cat::Esp32,
        icon: board(2.1, 15, true, true, 1),
        chip: "ESP32-WROOM-32 · Xtensa LX6 dual-core · 240 MHz",
        desc: "A placa de entrada mais popular da família. Wi-Fi 802.11 b/g/n, Bluetooth clássico + BLE, 30 ou 38 pinos e conversor USB-serial onboard. Ótima para prototipar IoT e laboratórios de segurança.",
    },
    Item {
        name: "ESP32-C3 DevKit",
        cat: Cat::Esp32,
        icon: board(1.9, 12, true, true, 1),
        chip: "ESP32-C3 · RISC-V 160 MHz · Wi-Fi + BLE 5",
        desc: "Núcleo RISC-V único, Wi-Fi e Bluetooth 5 (LE) com consumo baixo e custo menor. USB nativo para serial/JTAG, ideal para projetos pequenos e conectados.",
    },
    Item {
        name: "ESP32-C3 SuperMini",
        cat: Cat::Esp32,
        icon: board(1.3, 8, true, true, 1),
        chip: "ESP32-C3 · RISC-V · cerca de 22 x 18 mm",
        desc: "Versão minúscula do C3, do tamanho de um selo. Mesmo chip, USB-C e antena na própria placa; perfeita para wearables e dispositivos discretos.",
    },
    Item {
        name: "ESP32-S3 DevKitC",
        cat: Cat::Esp32,
        icon: board(2.2, 18, true, true, 1),
        chip: "ESP32-S3 · Xtensa LX7 dual-core · 240 MHz",
        desc: "Dois núcleos LX7, Wi-Fi, Bluetooth 5 (LE), USB OTG nativo e instruções de vetor para IA na borda. A mais completa para projetos de segurança, HID e câmeras.",
    },
    Item {
        name: "XIAO ESP32-C3",
        cat: Cat::Esp32,
        icon: board(1.25, 7, true, false, 1),
        chip: "Seeed Studio · ESP32-C3 · 21 x 17,8 mm",
        desc: "Placa ultracompacta da Seeed com carregador de bateria LiPo integrado, USB-C e conector para antena externa. Cabe em qualquer lugar.",
    },
    Item {
        name: "XIAO ESP32-S3",
        cat: Cat::Esp32,
        icon: board(1.25, 7, true, false, 1),
        chip: "Seeed Studio · ESP32-S3 · 8 MB PSRAM",
        desc: "Versão potente do XIAO: dual-core, 8 MB de PSRAM e 8 MB de flash, com suporte a bateria LiPo. A versão Sense ainda traz câmera e microfone.",
    },
    // ── Arduino ──
    Item {
        name: "Arduino Uno R3",
        cat: Cat::Arduino,
        icon: board(1.35, 14, true, false, 1),
        chip: "ATmega328P · 5 V · 16 MHz",
        desc: "O clássico para aprender eletrônica: 14 pinos digitais, 6 analógicos e USB-B. Compatível com milhares de shields e tutoriais.",
    },
    Item {
        name: "Arduino Uno R4 WiFi",
        cat: Cat::Arduino,
        icon: board(1.35, 14, true, false, 2),
        chip: "Renesas RA4M1 · 48 MHz + ESP32-S3",
        desc: "Evolução do Uno com Cortex-M4, matriz de LEDs 12x8 e Wi-Fi/Bluetooth via ESP32-S3. Mantém o formato e os 5 V do Uno.",
    },
    Item {
        name: "Arduino Nano",
        cat: Cat::Arduino,
        icon: board(2.6, 15, true, false, 1),
        chip: "ATmega328 · 5 V · 18 x 45 mm",
        desc: "O mesmo cérebro do Uno em formato de protoboard, com USB na própria placa. Pequeno e muito versátil.",
    },
    Item {
        name: "Arduino Mega 2560",
        cat: Cat::Arduino,
        icon: board(2.1, 26, true, false, 1),
        chip: "ATmega2560 · 54 I/O · 4 UARTs",
        desc: "Para projetos grandes: 54 pinos digitais, 16 analógicos, 256 KB de flash e quatro portas seriais.",
    },
    Item {
        name: "Arduino Leonardo",
        cat: Cat::Arduino,
        icon: board(1.35, 14, true, false, 1),
        chip: "ATmega32U4 · USB nativo",
        desc: "USB nativo permite que a placa seja reconhecida como teclado ou mouse (HID). Muito usada em automação e em estudos de segurança em laboratório.",
    },
    Item {
        name: "Arduino Micro",
        cat: Cat::Arduino,
        icon: board(2.6, 12, true, false, 1),
        chip: "ATmega32U4 · 48 x 18 mm",
        desc: "Um Leonardo em tamanho mini, com USB nativo. Perfeito para teclados e controles customizados.",
    },
    Item {
        name: "Arduino Pro Mini",
        cat: Cat::Arduino,
        icon: board(1.5, 12, false, false, 1),
        chip: "ATmega328P · 3,3 V ou 5 V",
        desc: "Sem porta USB e super leve; é programada por adaptador FTDI. Ótima para projetos embarcados de baixo consumo.",
    },
    Item {
        name: "Arduino Due",
        cat: Cat::Arduino,
        icon: board(2.0, 22, true, false, 1),
        chip: "AT91SAM3X8E · Cortex-M3 · 84 MHz",
        desc: "O primeiro Arduino com ARM de 32 bits. Opera em 3,3 V, tem 54 pinos de I/O e dois DACs. Para aplicações mais pesadas.",
    },
    Item {
        name: "Arduino Nano Every",
        cat: Cat::Arduino,
        icon: board(2.6, 15, true, false, 1),
        chip: "ATmega4809 · 5 V · 20 MHz",
        desc: "Nano modernizado, com mais memória e periféricos que o Nano clássico, no mesmo formato.",
    },
    // ── Displays ──
    Item {
        name: "OLED 0.96\" I2C",
        cat: Cat::Display,
        icon: Icon::Oled,
        chip: "SSD1306 · 128x64 · I2C",
        desc: "Tela monocromática de alto contraste e apenas 4 fios. Perfeita para status, menus e gráficos no ESP32.",
    },
    Item {
        name: "OLED 1.3\" I2C",
        cat: Cat::Display,
        icon: Icon::Oled,
        chip: "SH1106 · 128x64 · I2C",
        desc: "Tela maior com a mesma resolução. Atenção ao controlador SH1106 na hora de escolher a biblioteca.",
    },
    Item {
        name: "LCD 16x2 I2C",
        cat: Cat::Display,
        icon: Icon::Lcd { rows: 2 },
        chip: "HD44780 · 16 colunas x 2 linhas",
        desc: "Display de caracteres com backpack I2C. Simples de usar e legível mesmo sob luz forte. Muitos módulos operam em 5 V.",
    },
    Item {
        name: "LCD 20x4 I2C",
        cat: Cat::Display,
        icon: Icon::Lcd { rows: 4 },
        chip: "HD44780 · 20 colunas x 4 linhas",
        desc: "Mais linhas para mostrar menus e logs. Mesmo controle do 16x2 via I2C.",
    },
    // ── Módulos ──
    Item {
        name: "Leitor NFC PN532",
        cat: Cat::Modulo,
        icon: Icon::Nfc,
        chip: "PN532 · 13,56 MHz · I2C / SPI / HSU",
        desc: "Lê e escreve tags NFC/RFID e suporta modos de emulação de cartão. Muito usado em estudos de controle de acesso e segurança de cartões.",
    },
    Item {
        name: "Leitor NFC RC522",
        cat: Cat::Modulo,
        icon: Icon::Nfc,
        chip: "MFRC522 · 13,56 MHz · SPI",
        desc: "Módulo barato para ler cartões Mifare e tags. Ótimo para começar com RFID.",
    },
    Item {
        name: "CYD ESP32 Touch",
        cat: Cat::Modulo,
        icon: Icon::Cyd,
        chip: "ESP32-2432S028R · TFT 2.8\" 320x240 · touch",
        desc: "Cheap Yellow Display: ESP32 com tela touch colorida, slot microSD, LED RGB e sensor de luz numa placa só. Queridinho da comunidade para dashboards e ferramentas portáteis.",
    },
    Item {
        name: "Antena 2.4 GHz externa",
        cat: Cat::Modulo,
        icon: Icon::Antenna,
        chip: "Conector IPEX / U.FL · 2.4 GHz",
        desc: "Antena externa para placas ESP32 com conector U.FL. Melhora o alcance de Wi-Fi e Bluetooth em relação à antena impressa da placa.",
    },
    // ── Energia ──
    Item {
        name: "Bateria LiPo 300 mAh",
        cat: Cat::Energia,
        icon: Icon::Battery { mah: 300 },
        chip: "3,7 V · LiPo · 300 mAh",
        desc: "A menor da linha: ideal para wearables e placas como XIAO e SuperMini. Autonomia curta, mas leve e discreta.",
    },
    Item {
        name: "Bateria LiPo 400 mAh",
        cat: Cat::Energia,
        icon: Icon::Battery { mah: 400 },
        chip: "3,7 V · LiPo · 400 mAh",
        desc: "Boa para projetos compactos de baixo consumo e deep sleep. Use sempre um módulo de carga com proteção.",
    },
    Item {
        name: "Bateria LiPo 500 mAh",
        cat: Cat::Energia,
        icon: Icon::Battery { mah: 500 },
        chip: "3,7 V · LiPo · 500 mAh",
        desc: "Equilíbrio entre tamanho e autonomia para sensores e gadgets portáteis.",
    },
    Item {
        name: "Bateria LiPo 600 mAh",
        cat: Cat::Energia,
        icon: Icon::Battery { mah: 600 },
        chip: "3,7 V · LiPo · 600 mAh",
        desc: "Mais fôlego para ESP32 com display OLED ou Wi-Fi ocasional.",
    },
    Item {
        name: "Bateria LiPo 700 mAh",
        cat: Cat::Energia,
        icon: Icon::Battery { mah: 700 },
        chip: "3,7 V · LiPo · 700 mAh",
        desc: "Capacidade intermediária para projetos portáteis que ficam mais tempo ligados.",
    },
    Item {
        name: "Bateria LiPo 800 mAh",
        cat: Cat::Energia,
        icon: Icon::Battery { mah: 800 },
        chip: "3,7 V · LiPo · 800 mAh",
        desc: "A mais robusta desta lista: ideal para ferramentas portáteis com tela e Wi-Fi ativos por mais tempo.",
    },
    // ── PCB ──
    Item {
        name: "PCB Perfurada",
        cat: Cat::Pcb,
        icon: Icon::Perf,
        chip: "Perfboard · furos de 2,54 mm",
        desc: "Placa com furos padrão para soldar seus circuitos à mão. O passo do protótipo para o definitivo.",
    },
    Item {
        name: "PCB Customizada",
        cat: Cat::Pcb,
        icon: Icon::Pcb,
        chip: "Projeto em KiCad · fabricação sob encomenda",
        desc: "Placa feita sob medida, com trilhas, vias e serigrafia próprias. Ideal para transformar seu protótipo em um produto de verdade.",
    },
    Item {
        name: "Teclado Membrana 4x3",
        cat: Cat::Modulo,
        icon: Icon::Pcb,
        chip: "Matriz 4x3 · 12 teclas · 7 pinos",
        desc: "Teclado matricial flexível e autoadesivo com 12 teclas (0 a 9, * e #). Ocupa pouco espaço e é ótimo para senhas, travas e menus em projetos com ESP32 ou Arduino.",
    },
    Item {
        name: "WiFi LoRa 32 V3 (ESP32-S3)",
        cat: Cat::Esp32,
        icon: Icon::Oled,
        chip: "ESP32-S3 + LoRa SX1262 · OLED 0,96\" · USB-C",
        desc: "Placa com ESP32-S3, rádio LoRa de longo alcance, Wi-Fi, Bluetooth LE, display OLED e conector de bateria. Vem com antena e pinos para soldar; ótima para redes de sensores e comunicação sem internet.",
    },
    Item {
        name: "Módulo Micro SD (TF Card)",
        cat: Cat::Modulo,
        icon: Icon::Pcb,
        chip: "Interface SPI · cartão microSD / TF",
        desc: "Adaptador para ler e gravar cartões microSD via SPI. Serve para guardar logs, capturas e arquivos em projetos de ESP32 e Arduino.",
    },
    Item {
        name: "Sensor de Chuva",
        cat: Cat::Modulo,
        icon: Icon::Pcb,
        chip: "Placa de gotas + módulo comparador · saída analógica e digital",
        desc: "Detecta gotas de água na placa de trilhas e entrega o sinal pelo módulo, com ajuste de sensibilidade por trimpot. Usado em estações meteorológicas e automações contra chuva.",
    },
    Item {
        name: "Módulo nRF24L01 com Antena",
        cat: Cat::Modulo,
        icon: Icon::Pcb,
        chip: "nRF24L01 · 2,4 GHz · SPI · antena SMA",
        desc: "Rádio de 2,4 GHz para comunicação sem fio entre placas, com antena externa para ganhar alcance (o anúncio indica até 1000 m em campo aberto). Muito usado em controles remotos e redes de sensores.",
    },
    Item {
        name: "Kit Resistor 10k 1/4W",
        cat: Cat::Comp,
        icon: Icon::Perf,
        chip: "10 kΩ · 1/4 W · 5% · carbono",
        desc: "Kit de 200 resistores de carbono de 10 kΩ. É o valor clássico para pull-up e pull-down em botões e entradas digitais.",
    },
    Item {
        name: "Jumpers Rígidos Coloridos",
        cat: Cat::Comp,
        icon: Icon::Perf,
        chip: "Kit com caixa organizadora · para protoboard",
        desc: "Fios rígidos pré-cortados em vários comprimentos e cores, já dobrados para a protoboard. Deixam a montagem limpa e fácil de revisar.",
    },
    Item {
        name: "Protoboard 400 Pontos",
        cat: Cat::Comp,
        icon: Icon::Perf,
        chip: "400 furos · 8,5 x 5,5 cm",
        desc: "Placa de ensaio compacta para montar circuitos sem solda. Boa para testes rápidos com ESP32, sensores e módulos.",
    },
    Item {
        name: "Protoboard 830 Pontos",
        cat: Cat::Comp,
        icon: Icon::Perf,
        chip: "830 furos · trilhas de alimentação duplas",
        desc: "Protoboard grande para circuitos maiores, com trilhas de alimentação dos dois lados. Cabe uma placa de desenvolvimento e vários módulos juntos.",
    },
];

// ───────────────────────────── Utilidades ─────────────────────────────

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    a.lerp_to_gamma(b, t.clamp(0.0, 1.0))
}

/// Tom de cinza multiplicado pelo "fade" (0..1). Como tudo é preto e branco,
/// multiplicar o cinza equivale a um fade-in sobre o fundo preto.
fn gray(v: u8, fade: f32) -> Color32 {
    Color32::from_gray((v as f32 * fade.clamp(0.0, 1.0)) as u8)
}

fn st(w: f32, c: Color32) -> Stroke {
    Stroke::new(w, c)
}

fn fit(rect: Rect, aspect: f32, pad: f32) -> Rect {
    let r = rect.shrink(pad);
    let (mut w, mut h) = (r.width(), r.height());
    if w / h > aspect {
        w = h * aspect;
    } else {
        h = w / aspect;
    }
    Rect::from_center_size(r.center(), vec2(w, h))
}

fn arc(p: &Painter, c: Pos2, rad: f32, a0: f32, a1: f32, stroke: Stroke) {
    let n = 14;
    let mut last = pos2(c.x + rad * a0.cos(), c.y + rad * a0.sin());
    for i in 1..=n {
        let a = a0 + (a1 - a0) * i as f32 / n as f32;
        let q = pos2(c.x + rad * a.cos(), c.y + rad * a.sin());
        p.line_segment([last, q], stroke);
        last = q;
    }
}

fn along(pts: &[Pos2], t: f32) -> Pos2 {
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

// ───────────────────────────── Ícones desenhados em código ─────────────────────────────

fn draw_icon(p: &Painter, rect: Rect, icon: Icon, hov: f32, time: f64, fade: f32) {
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
        Icon::Antenna => {
            let cx = rect.center().x;
            let base = pos2(cx, rect.bottom() - 34.0);
            let top = pos2(cx, rect.top() + 46.0);
            p.line_segment([base, top], st(2.4, c(235)));
            p.circle_filled(top, 4.5, c(255));
            let blk = Rect::from_center_size(base + vec2(0.0, 6.0), vec2(30.0, 14.0));
            p.rect_filled(blk, 2.0, c(14));
            p.rect_stroke(blk, 2.0, st(1.4, c(220)));
            let e0 = blk.right_center();
            let e1 = e0 + vec2(26.0, 0.0);
            p.line_segment([e0, e1], st(1.6, c(150)));
            p.rect_stroke(
                Rect::from_center_size(e1 + vec2(4.0, 0.0), vec2(8.0, 8.0)),
                1.0,
                st(1.2, c(200)),
            );
            let amp = 0.3 + 0.7 * hov;
            for k in 0..3 {
                let ph = (t * 0.9 + k as f32 / 3.0) % 1.0;
                let rad = 14.0 + 30.0 * ph;
                let col = c((200.0 * (1.0 - ph) * amp) as u8);
                arc(p, top, rad, PI - 0.9, PI + 0.9, st(1.6, col));
                arc(p, top, rad, -0.9, 0.9, st(1.6, col));
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

// ───────────────────────────── Widgets ─────────────────────────────

fn chip(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    let font = FontId::monospace(13.0);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), Color32::WHITE);
    let size = vec2(galley.size().x + 26.0, 30.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let sel = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("sel"), selected, 0.2);
    let hov = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("hov"), resp.hovered(), 0.12);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let fill = mix(
        mix(Color32::BLACK, Color32::from_gray(30), hov),
        Color32::WHITE,
        sel,
    );
    let txt = mix(Color32::from_gray(210), Color32::BLACK, sel);
    let p = ui.painter();
    p.rect_filled(rect, 4.0, fill);
    p.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, mix(Color32::from_gray(70), Color32::WHITE, hov.max(sel))),
    );
    p.text(rect.center(), Align2::CENTER_CENTER, label, font, txt);
    resp.clicked()
}

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

fn paint_img(p: &Painter, area: Rect, tex: &TextureHandle, e: f32, hov: f32) {
    let panel = area.shrink2(vec2(12.0, 10.0));
    p.rect_filled(panel, 6.0, gray(244, e));
    let s = tex.size_vec2();
    let inner = panel.shrink(6.0);
    let k = (inner.width() / s.x).min(inner.height() / s.y) * (1.0 + 0.06 * hov);
    let r = Rect::from_center_size(inner.center(), s * k);
    p.with_clip_rect(panel).image(
        tex.id(),
        r,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        gray(255, e),
    );
}

fn draw_avatar(p: &Painter, c: Pos2, r: f32, tex: Option<&TextureHandle>, letter: &str, fade: f32) {
    match tex {
        Some(t) => {
            let mut m = egui::Mesh::with_texture(t.id());
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
            p.add(egui::Shape::mesh(m));
        }
        None => {
            p.circle_filled(c, r, gray(30, fade));
            p.text(
                c,
                Align2::CENTER_CENTER,
                letter,
                FontId::monospace(r * 0.9),
                gray(240, fade),
            );
        }
    }
    p.circle_stroke(c, r, st(1.5, gray(235, fade)));
}

#[allow(clippy::too_many_arguments)]
fn paint_card(
    p: &Painter,
    r: Rect,
    it: &Item,
    idx: usize,
    hov: f32,
    time: f64,
    e: f32,
    tex: Option<&TextureHandle>,
) {
    let c = |v: u8| gray(v, e);
    p.rect_filled(r, 8.0, mix(c(8), c(20), hov));
    p.rect_stroke(r, 8.0, Stroke::new(1.0 + 0.8 * hov, mix(c(55), c(255), hov)));

    // cantos estilo "mira"
    let bl = 9.0 + 5.0 * hov;
    let tick = st(1.5, mix(c(80), c(255), hov));
    for (cx, cy, dx, dy) in [
        (r.left() + 7.0, r.top() + 7.0, 1.0, 1.0),
        (r.right() - 7.0, r.top() + 7.0, -1.0, 1.0),
        (r.left() + 7.0, r.bottom() - 7.0, 1.0, -1.0),
        (r.right() - 7.0, r.bottom() - 7.0, -1.0, -1.0),
    ] {
        p.line_segment([pos2(cx, cy), pos2(cx + dx * bl, cy)], tick);
        p.line_segment([pos2(cx, cy), pos2(cx, cy + dy * bl)], tick);
    }

    let area = Rect::from_min_size(r.min, vec2(CARD_W, ICON_H));
    match tex {
        Some(t) => paint_img(p, area, t, e, hov),
        None => draw_icon(p, area, it.icon, hov, time, e),
    }
    p.line_segment(
        [
            pos2(r.left() + 14.0, r.top() + ICON_H),
            pos2(r.right() - 14.0, r.top() + ICON_H),
        ],
        st(1.0, c(35)),
    );
    p.text(
        pos2(r.left() + 18.0, r.top() + ICON_H + 22.0),
        Align2::LEFT_CENTER,
        it.name,
        FontId::proportional(16.5),
        c(245),
    );
    p.text(
        pos2(r.left() + 18.0, r.top() + ICON_H + 46.0),
        Align2::LEFT_CENTER,
        it.cat.label(),
        FontId::monospace(11.5),
        c(120),
    );
    p.text(
        pos2(r.right() - 18.0, r.top() + ICON_H + 46.0),
        Align2::RIGHT_CENTER,
        "ABRIR >",
        FontId::monospace(11.5),
        c((230.0 * hov) as u8),
    );
    p.text(
        pos2(r.left() + 16.0, r.top() + 18.0),
        Align2::LEFT_CENTER,
        format!("{:02}", idx + 1),
        FontId::monospace(11.0),
        c(95),
    );
}

/// Retorna o retângulo do card se ele foi clicado.
fn card(
    ui: &mut egui::Ui,
    it: &Item,
    idx: usize,
    order: usize,
    t0: f64,
    active: bool,
    tex: Option<&TextureHandle>,
) -> Option<Rect> {
    let time = ui.input(|i| i.time);
    let k = (((time - t0) - order as f64 * 0.04) / 0.5).clamp(0.0, 1.0) as f32;
    let e = easing::cubic_out(k);
    let (rect, resp) = ui.allocate_exact_size(vec2(CARD_W, CARD_H), Sense::click());
    if k <= 0.0 || active {
        return None;
    }
    let hov = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("hov"), resp.hovered(), 0.18);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let r = rect.translate(vec2(0.0, -6.0 * hov + 24.0 * (1.0 - e)));
    paint_card(ui.painter(), r, it, idx, hov, time, e, tex);
    if resp.clicked() {
        Some(rect)
    } else {
        None
    }
}

fn big_button(ui: &mut egui::Ui, label: &str, w: f32, fade: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 44.0), Sense::click());
    let hov = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.2);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let p = ui.painter();
    p.rect_stroke(rect, 4.0, st(1.5, gray(235, fade)));
    let fw = rect.width() * easing::cubic_out(hov);
    let filled = Rect::from_min_size(rect.min, vec2(fw, rect.height()));
    p.rect_filled(filled, 4.0, gray(245, fade));
    let font = FontId::monospace(15.0);
    p.text(rect.center(), Align2::CENTER_CENTER, label, font.clone(), gray(240, fade));
    p.with_clip_rect(filled)
        .text(rect.center(), Align2::CENTER_CENTER, label, font, Color32::BLACK);
    resp.clicked()
}

fn menu_item(ui: &mut egui::Ui, label: &str, w: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 36.0), Sense::click());
    let hov = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.15);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let p = ui.painter();
    p.rect_filled(rect, 3.0, mix(Color32::from_gray(8), Color32::WHITE, hov));
    p.text(
        rect.left_center() + vec2(12.0 + 4.0 * hov, 0.0),
        Align2::LEFT_CENTER,
        label,
        FontId::monospace(13.0),
        mix(Color32::from_gray(220), Color32::BLACK, hov),
    );
    resp.clicked()
}

fn draw_background(ctx: &egui::Context, time: f64) {
    let r = ctx.screen_rect();
    let p = ctx.layer_painter(egui::LayerId::background());
    let gap = 44.0_f32;
    let off = ((time * 6.0) % gap as f64) as f32;
    let line = st(1.0, Color32::from_gray(15));
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
    // varredura
    let sy = r.top() + ((time * 90.0) % (r.height() as f64 + 160.0)) as f32 - 80.0;
    for k in 0..10 {
        let v = (30 - k * 3) as u8;
        p.line_segment(
            [pos2(r.left(), sy - k as f32 * 3.0), pos2(r.right(), sy - k as f32 * 3.0)],
            st(1.0, Color32::from_gray(v)),
        );
    }
}

// ───────────────────────────── App ─────────────────────────────

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Login,
    Register,
}

struct Sel {
    idx: usize,
    rect: Rect,
    t0: f64,
}

struct CyberDesk {
    store: auth::Store,
    user: Option<String>,
    mode: Mode,
    f_user: String,
    f_pass: String,
    f_pass2: String,
    err: Option<(String, f64)>,
    fails: u32,
    locked_until: f64,
    pending_avatar: Option<(Vec<u8>, TextureHandle)>,
    avatar: Option<TextureHandle>,
    menu_open: bool,
    login_t0: f64,
    cat: Option<Cat>,
    query: String,
    last_key: (Option<Cat>, String),
    grid_t0: f64,
    sel: Option<Sel>,
    open: bool,
    imgs: Vec<Option<TextureHandle>>,
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
        Self {
            store: auth::Store::load(),
            user: None,
            mode: Mode::Login,
            f_user: String::new(),
            f_pass: String::new(),
            f_pass2: String::new(),
            err: None,
            fails: 0,
            locked_until: 0.0,
            pending_avatar: None,
            avatar: None,
            menu_open: false,
            login_t0: 0.0,
            cat: None,
            query: String::new(),
            last_key: (None, String::new()),
            grid_t0: 0.0,
            sel: None,
            open: false,
            imgs,
        }
    }

    fn fail(&mut self, msg: &str, time: f64) {
        self.err = Some((msg.to_string(), time));
    }

    fn load_avatar(&mut self, ctx: &egui::Context) {
        self.avatar = self.user.as_ref().and_then(|u| {
            self.store
                .avatar_bytes(u)
                .and_then(|b| make_tex(ctx, &format!("avatar-{u}"), &b, 256, true))
        });
    }

    fn apply_avatar(&mut self, ctx: &egui::Context, bytes: Vec<u8>, time: f64) {
        if bytes.len() > 20 * 1024 * 1024 {
            self.fail("Imagem muito grande (máx. 20 MB)", time);
            return;
        }
        if let Some(u) = self.user.clone() {
            match self.store.set_avatar(&u, &bytes) {
                Ok(()) => self.load_avatar(ctx),
                Err(e) => self.fail(&e, time),
            }
        } else if self.mode == Mode::Register {
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
        let result = match self.mode {
            Mode::Login => self.store.login(&self.f_user, &self.f_pass),
            Mode::Register => {
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
            }
        };
        match result {
            Ok(name) => {
                self.user = Some(name);
                self.f_user.clear();
                self.f_pass.clear();
                self.f_pass2.clear();
                self.err = None;
                self.fails = 0;
                self.pending_avatar = None;
                self.mode = Mode::Login;
                self.login_t0 = time;
                self.grid_t0 = time;
                self.load_avatar(ctx);
            }
            Err(e) => {
                if self.mode == Mode::Login {
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
        self.user = None;
        self.avatar = None;
        self.menu_open = false;
        self.open = false;
        self.sel = None;
        self.mode = Mode::Login;
        self.err = None;
        self.login_t0 = time;
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
        let mut do_submit = ctx.input(|i| i.key_pressed(egui::Key::Enter));
        let mut pick = false;
        let mut switch = false;

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::BLACK))
            .show(ctx, |ui| {
                draw_background(ui.ctx(), time);
                let w = 380.0;
                let reg = self.mode == Mode::Register;
                let h = if reg { 660.0 } else { 470.0 };
                let area = ui.max_rect();
                let rect = Rect::from_center_size(
                    area.center() + vec2(shake, (1.0 - e) * 30.0),
                    vec2(w, h),
                );
                ui.allocate_new_ui(
                    egui::UiBuilder::new()
                        .max_rect(rect)
                        .layout(egui::Layout::top_down(egui::Align::Center)),
                    |ui| {
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(typed_title(time - self.login_t0))
                                .monospace()
                                .size(40.0)
                                .color(gray(255, e)),
                        );
                        ui.label(
                            RichText::new(if reg {
                                "// criar conta local"
                            } else {
                                "// acesso restrito · login local"
                            })
                            .monospace()
                            .size(13.0)
                            .color(gray(150, e)),
                        );
                        ui.add_space(18.0);

                        if reg {
                            let (ar, resp) =
                                ui.allocate_exact_size(vec2(100.0, 100.0), Sense::click());
                            let hv = ui
                                .ctx()
                                .animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.2);
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            let ctr = ar.center();
                            let p = ui.painter();
                            match &self.pending_avatar {
                                Some((_, tex)) => draw_avatar(p, ctr, 42.0, Some(tex), "", e),
                                None => {
                                    p.circle_filled(ctr, 42.0, gray(14, e));
                                    p.text(
                                        ctr,
                                        Align2::CENTER_CENTER,
                                        "+",
                                        FontId::monospace(34.0),
                                        gray(200, e),
                                    );
                                }
                            }
                            let segs = 24;
                            for i in 0..segs {
                                let a0 = i as f32 / segs as f32 * 2.0 * PI
                                    + time as f32 * (0.3 + 1.5 * hv);
                                arc(p, ctr, 47.0 + 2.0 * hv, a0, a0 + 0.16, st(1.5, gray(200, e)));
                            }
                            if resp.clicked() {
                                pick = true;
                            }
                            ui.label(
                                RichText::new("foto de perfil (opcional) · clique ou arraste")
                                    .monospace()
                                    .size(11.5)
                                    .color(gray(130, e)),
                            );
                            ui.add_space(12.0);
                        }

                        ui.add(
                            egui::TextEdit::singleline(&mut self.f_user)
                                .hint_text("usuário")
                                .font(FontId::monospace(15.0))
                                .margin(vec2(10.0, 9.0))
                                .desired_width(w),
                        );
                        ui.add_space(10.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut self.f_pass)
                                .password(true)
                                .hint_text("senha")
                                .font(FontId::monospace(15.0))
                                .margin(vec2(10.0, 9.0))
                                .desired_width(w),
                        );
                        if reg {
                            ui.add_space(10.0);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.f_pass2)
                                    .password(true)
                                    .hint_text("confirmar senha")
                                    .font(FontId::monospace(15.0))
                                    .margin(vec2(10.0, 9.0))
                                    .desired_width(w),
                            );
                        }

                        ui.add_space(8.0);
                        let msg = match &self.err {
                            Some((m, _)) => format!("! {m}"),
                            None if reg => "// senha: mínimo 6 caracteres".to_string(),
                            None => String::new(),
                        };
                        let is_err = self.err.is_some();
                        ui.label(
                            RichText::new(msg)
                                .monospace()
                                .size(12.5)
                                .color(gray(if is_err { 255 } else { 120 }, e)),
                        );
                        ui.add_space(8.0);

                        let label = if reg { "CRIAR CONTA" } else { "ENTRAR" };
                        if big_button(ui, label, w, e) {
                            do_submit = true;
                        }
                        ui.add_space(14.0);
                        let toggle = if reg {
                            "já tenho conta · entrar"
                        } else {
                            "não tenho conta · criar agora"
                        };
                        let r = ui.add(
                            egui::Label::new(
                                RichText::new(toggle)
                                    .monospace()
                                    .size(13.0)
                                    .color(gray(180, e))
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
            });

        // arrastar imagem por cima da janela
        if ctx.input(|i| !i.raw.hovered_files.is_empty()) && self.mode == Mode::Register {
            let p = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("drop")));
            let r = ctx.screen_rect().shrink(14.0);
            p.rect_filled(r, 8.0, Color32::from_black_alpha(190));
            p.rect_stroke(r, 8.0, st(2.0, Color32::WHITE));
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                "SOLTE A IMAGEM AQUI",
                FontId::monospace(22.0),
                Color32::WHITE,
            );
        }

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
        }
        if do_submit {
            self.submit(ctx, time);
        }
    }

    // ───── Catálogo ─────
    fn content(&mut self, ui: &mut egui::Ui, time: f64) {
        let tt = time - self.login_t0;
        let (tr, _) = ui.allocate_exact_size(vec2(ui.available_width(), 56.0), Sense::hover());
        let p = ui.painter().clone();
        let txt = typed_title(tt);
        let pos = pos2(tr.left(), tr.center().y);
        let font = FontId::monospace(44.0);
        if tt > 1.2 && (time % 6.0) < 0.16 {
            p.text(pos + vec2(4.0, -2.0), Align2::LEFT_CENTER, &txt, font.clone(), Color32::from_gray(110));
            p.text(pos + vec2(-3.0, 2.0), Align2::LEFT_CENTER, &txt, font.clone(), Color32::from_gray(60));
        }
        p.text(pos, Align2::LEFT_CENTER, &txt, font, Color32::WHITE);

        ui.label(
            RichText::new("// catálogo de hardware · cibersegurança · esp32 · arduino")
                .monospace()
                .size(13.0)
                .color(Color32::from_gray(150)),
        );
        ui.add_space(10.0);
        let (lr, _) = ui.allocate_exact_size(vec2(ui.available_width(), 2.0), Sense::hover());
        let k = easing::cubic_out(((tt - 0.2) / 1.4).clamp(0.0, 1.0) as f32);
        ui.painter().line_segment(
            [lr.left_center(), pos2(lr.left() + lr.width() * k, lr.center().y)],
            Stroke::new(1.5, Color32::WHITE),
        );

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(">").monospace().size(18.0));
            ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text("buscar item...")
                    .font(FontId::monospace(14.0))
                    .desired_width(300.0),
            );
        });

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            let cats: [(Option<Cat>, &str); 8] = [
                (None, "TODOS"),
                (Some(Cat::Esp32), "ESP32"),
                (Some(Cat::Arduino), "ARDUINO"),
                (Some(Cat::Display), "DISPLAYS"),
                (Some(Cat::Modulo), "MÓDULOS"),
                (Some(Cat::Energia), "ENERGIA"),
                (Some(Cat::Pcb), "PCB"),
                (Some(Cat::Comp), "COMPONENTES"),
            ];
            for (c, label) in cats {
                let n = match c {
                    None => ITEMS.len(),
                    Some(cc) => ITEMS.iter().filter(|i| i.cat == cc).count(),
                };
                if chip(ui, &format!("{label} {n}"), self.cat == c) {
                    self.cat = c;
                }
            }
        });

        let key = (self.cat, self.query.clone());
        if key != self.last_key {
            self.grid_t0 = time;
            self.last_key = key;
        }
        let q = self.query.to_lowercase();
        let list: Vec<usize> = ITEMS
            .iter()
            .enumerate()
            .filter(|(_, it)| {
                self.cat.map_or(true, |c| c == it.cat)
                    && (q.is_empty()
                        || it.name.to_lowercase().contains(&q)
                        || it.desc.to_lowercase().contains(&q))
            })
            .map(|(i, _)| i)
            .collect();

        ui.add_space(6.0);
        ui.label(
            RichText::new(format!("// {} resultado(s)", list.len()))
                .monospace()
                .size(12.0)
                .color(Color32::from_gray(120)),
        );
        ui.add_space(8.0);

        let t0 = self.grid_t0;
        let active_idx = if self.open {
            self.sel.as_ref().map(|s| s.idx)
        } else {
            None
        };
        let imgs = &self.imgs;
        let mut clicked: Option<(usize, Rect)> = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(4.0);
                if list.is_empty() {
                    ui.label(
                        RichText::new("// nenhum item encontrado")
                            .monospace()
                            .size(14.0)
                            .color(Color32::from_gray(140)),
                    );
                }
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(18.0, 18.0);
                    for (order, &idx) in list.iter().enumerate() {
                        let active = active_idx == Some(idx);
                        if let Some(r) = card(
                            ui,
                            &ITEMS[idx],
                            idx,
                            order,
                            t0,
                            active,
                            imgs[idx].as_ref(),
                        ) {
                            clicked = Some((idx, r));
                        }
                    }
                });
                ui.add_space(24.0);
            });
        if let Some((idx, rect)) = clicked {
            self.sel = Some(Sel { idx, rect, t0: time });
            self.open = true;
            self.menu_open = false;
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
        let tt = ((time - self.login_t0) / 0.8).clamp(0.0, 1.0) as f32;
        let e = easing::cubic_out(tt);

        egui::Area::new(Id::new("profile_chip"))
            .order(Order::Middle)
            .anchor(Align2::RIGHT_TOP, vec2(-32.0, 22.0))
            .show(ctx, |ui| {
                let w = 60.0 + name.len() as f32 * 8.5;
                let (rect, resp) = ui.allocate_exact_size(vec2(w, 44.0), Sense::click());
                let hv = ctx.animate_bool_with_time(resp.id.with("h"), resp.hovered(), 0.15);
                if resp.hovered() {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let p = ui.painter();
                p.rect_filled(rect, 22.0, mix(gray(8, e), gray(26, e), hv));
                p.rect_stroke(rect, 22.0, st(1.0, mix(gray(70, e), gray(255, e), hv)));
                draw_avatar(
                    p,
                    pos2(rect.left() + 22.0, rect.center().y),
                    17.0,
                    avatar.as_ref(),
                    &letter,
                    e,
                );
                p.text(
                    pos2(rect.left() + 48.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    &name,
                    FontId::monospace(14.0),
                    gray(240, e),
                );
                if resp.clicked() {
                    toggle = true;
                }
            });

        let mt = ctx.animate_bool_with_time(Id::new("menu_t"), self.menu_open, 0.18);
        if mt > 0.0 {
            egui::Area::new(Id::new("profile_menu"))
                .order(Order::Middle)
                .anchor(Align2::RIGHT_TOP, vec2(-32.0, 74.0 - 8.0 * (1.0 - mt)))
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(gray(6, mt))
                        .stroke(st(1.0, gray(200, mt)))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::same(6.0))
                        .show(ui, |ui| {
                            if menu_item(ui, "TROCAR FOTO", 190.0) {
                                change_photo = true;
                            }
                            if menu_item(ui, "SAIR", 190.0) {
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

    // ───── Botão destacado + popup ao lado + fundo fosco ─────
    fn spotlight(&mut self, ctx: &egui::Context, time: f64) {
        let t = ctx.animate_bool_with_time(Id::new("panel_t"), self.open, 0.35);
        let Some(sel) = &self.sel else { return };
        if t <= 0.0 {
            return;
        }
        let (idx, card_rect, t0) = (sel.idx, sel.rect, sel.t0);
        let it = &ITEMS[idx];
        let e = easing::cubic_out(t);
        let screen = ctx.screen_rect();
        let mut close = false;

        // 1) fundo escurecido e "fosco" (clicar fora fecha)
        egui::Area::new(Id::new("dim"))
            .order(Order::Foreground)
            .fixed_pos(screen.min)
            .constrain(false)
            .show(ctx, |ui| {
                let (r, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
                let p = ui.painter();
                p.rect_filled(r, 0.0, Color32::from_black_alpha((205.0 * e) as u8));
                let line = Color32::from_white_alpha((9.0 * e) as u8);
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
                12.0,
                st(1.2, gray((190.0 * (1.0 - ph) * e) as u8, 1.0)),
            );
        }
        paint_card(&spot, scaled, it, idx, 1.0, time, 1.0, self.imgs[idx].as_ref());

        // 3) popup ao lado do card (direita; se faltar espaço, esquerda)
        let pw = 350.0_f32;
        let ph = 310.0_f32;
        let right = scaled.right() + 22.0 + pw <= screen.right() - 16.0;
        let base_x = if right {
            scaled.right() + 22.0
        } else {
            scaled.left() - 22.0 - pw
        };
        let dir = if right { 1.0 } else { -1.0 };
        let x = base_x - dir * 48.0 * (1.0 - e);
        let y = scaled
            .top()
            .clamp(16.0, (screen.bottom() - ph - 16.0).max(16.0));

        // conector animado entre o card e o popup
        let cy = scaled.center().y.clamp(y + 20.0, y + ph - 20.0);
        let (ax, bx) = if right {
            (scaled.right(), base_x)
        } else {
            (scaled.left(), base_x + pw)
        };
        let bx_now = ax + (bx - ax) * e;
        spot.line_segment([pos2(ax, cy), pos2(bx_now, cy)], st(1.5, gray(235, e)));
        let dot = ax + (bx_now - ax) * (((time * 0.8) % 1.0) as f32);
        spot.circle_filled(pos2(dot, cy), 3.0, gray(255, e));

        let sel_t0 = t0;
        egui::Area::new(Id::new("detail"))
            .order(Order::Tooltip)
            .fixed_pos(pos2(x, y))
            .constrain(false)
            .show(ctx, |ui| {
                let c = |v: u8| gray(v, e);
                egui::Frame::none()
                    .fill(c(5))
                    .stroke(st(1.0, c(220)))
                    .rounding(8.0)
                    .inner_margin(egui::Margin::same(20.0))
                    .show(ui, |ui| {
                        ui.set_width(pw - 40.0);
                        ui.set_min_height(ph - 40.0);

                        let (bar, _) =
                            ui.allocate_exact_size(vec2(pw - 40.0, 24.0), Sense::hover());
                        ui.painter().text(
                            bar.left_center(),
                            Align2::LEFT_CENTER,
                            "// DETALHES",
                            FontId::monospace(12.0),
                            c(130),
                        );
                        let xr = Rect::from_center_size(
                            bar.right_center() - vec2(12.0, 0.0),
                            vec2(24.0, 24.0),
                        );
                        let resp = ui.interact(xr, Id::new("close_x"), Sense::click());
                        let hv = ctx.animate_bool_with_time(Id::new("close_h"), resp.hovered(), 0.12);
                        if resp.hovered() {
                            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        let xc = mix(c(150), c(255), hv);
                        let d = 6.0 + 2.0 * hv;
                        let ctr = xr.center();
                        ui.painter().line_segment([ctr + vec2(-d, -d), ctr + vec2(d, d)], st(1.8, xc));
                        ui.painter().line_segment([ctr + vec2(-d, d), ctr + vec2(d, -d)], st(1.8, xc));
                        if resp.clicked() {
                            close = true;
                        }

                        ui.add_space(10.0);
                        ui.label(RichText::new(it.name).size(26.0).strong().color(c(255)));
                        let (ln, _) = ui.allocate_exact_size(vec2(pw - 40.0, 3.0), Sense::hover());
                        let lk = easing::cubic_out(((time - sel_t0) / 0.7).clamp(0.0, 1.0) as f32);
                        ui.painter().line_segment(
                            [ln.left_center(), pos2(ln.left() + ln.width() * lk, ln.center().y)],
                            st(2.0, c(255)),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(format!("[ {} ]", it.cat.label()))
                                .monospace()
                                .size(12.0)
                                .color(c(150)),
                        );
                        ui.add_space(12.0);
                        ui.label(RichText::new(it.chip).monospace().size(12.0).color(c(170)));
                        ui.add_space(10.0);
                        ui.label(RichText::new(it.desc).size(15.0).color(c(220)));
                    });
            });

        if close {
            self.open = false;
        }
    }
}

impl eframe::App for CyberDesk {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let time = ctx.input(|i| i.time);
        ctx.request_repaint_after(Duration::from_millis(16));

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

        if self.user.is_none() {
            self.login_screen(ctx, time);
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.menu_open {
                self.menu_open = false;
            } else {
                self.open = false;
            }
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::BLACK)
                    .inner_margin(egui::Margin::symmetric(32.0, 24.0)),
            )
            .show(ctx, |ui| {
                draw_background(ui.ctx(), time);
                self.content(ui, time);
            });

        self.profile_chip(ctx, time);
        self.spotlight(ctx, time);
    }
}

fn setup_style(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = Color32::BLACK;
    v.window_fill = Color32::BLACK;
    v.extreme_bg_color = Color32::from_gray(8);
    v.selection.bg_fill = Color32::from_gray(210);
    v.selection.stroke = Stroke::new(1.0, Color32::BLACK);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_gray(215));
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_gray(220));
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_gray(70));
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_gray(170));
    v.widgets.active.bg_stroke = Stroke::new(1.0, Color32::WHITE);
    ctx.set_visuals(v);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1240.0, 780.0])
            .with_min_inner_size([760.0, 520.0])
            .with_title("CyberDesk")
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "CyberDesk",
        options,
        Box::new(|cc| {
            setup_style(&cc.egui_ctx);
            Ok(Box::new(CyberDesk::new(&cc.egui_ctx)))
        }),
    )
}
