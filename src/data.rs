//! Catálogo de hardware: categorias, níveis e a lista de itens.
//!
//! ATENÇÃO: o nome do item gera o nome da imagem em `assets/items/`
//! (ex.: "ESP32 DevKit V1" -> `esp32-devkit-v1.png`). Ao renomear um item,
//! renomeie também a imagem.

use eframe::egui::Color32;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cat {
    Esp32,
    Arduino,
    Display,
    Modulo,
    Energia,
    Pcb,
    Comp,
}

pub const CATS: [Cat; 7] = [
    Cat::Esp32,
    Cat::Arduino,
    Cat::Display,
    Cat::Modulo,
    Cat::Energia,
    Cat::Pcb,
    Cat::Comp,
];

impl Cat {
    pub fn label(self) -> &'static str {
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

    pub fn plural(self) -> &'static str {
        match self {
            Cat::Esp32 => "ESP32",
            Cat::Arduino => "ARDUINO",
            Cat::Display => "DISPLAYS",
            Cat::Modulo => "MÓDULOS",
            Cat::Energia => "ENERGIA",
            Cat::Pcb => "PCBs",
            Cat::Comp => "COMPONENTES",
        }
    }

    /// Cor de destaque da categoria.
    pub fn color(self) -> Color32 {
        match self {
            Cat::Esp32 => Color32::from_rgb(34, 211, 238),
            Cat::Arduino => Color32::from_rgb(45, 212, 160),
            Cat::Display => Color32::from_rgb(167, 139, 250),
            Cat::Modulo => Color32::from_rgb(251, 191, 36),
            Cat::Energia => Color32::from_rgb(163, 230, 53),
            Cat::Pcb => Color32::from_rgb(244, 114, 182),
            Cat::Comp => Color32::from_rgb(251, 146, 60),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Iniciante,
    Intermediario,
    Avancado,
}

impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Level::Iniciante => "INICIANTE",
            Level::Intermediario => "INTERMEDIÁRIO",
            Level::Avancado => "AVANÇADO",
        }
    }

    /// 1 a 3 (quantas barrinhas acendem).
    pub fn bars(self) -> usize {
        match self {
            Level::Iniciante => 1,
            Level::Intermediario => 2,
            Level::Avancado => 3,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Icon {
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

pub struct Item {
    pub name: &'static str,
    pub cat: Cat,
    pub icon: Icon,
    pub level: Level,
    /// Especificações separadas por " · " (viram etiquetas no painel).
    pub chip: &'static str,
    /// Frase curta que aparece no card (até ~30 caracteres).
    pub tagline: &'static str,
    pub desc: &'static str,
    /// Dica prática de bancada (aparece em destaque no painel de detalhes).
    pub tip: &'static str,
    /// Usos comuns (mostrados como lista no painel).
    pub uses: &'static [&'static str],
}

use Level::{Avancado as Adv, Iniciante as Ini, Intermediario as Mid};

pub const ITEMS: &[Item] = &[
    // ── ESP32 ──
    Item {
        name: "ESP32 DevKit V1",
        cat: Cat::Esp32,
        icon: board(2.1, 15, true, true, 1),
        level: Ini,
        chip: "ESP32-WROOM-32 · Xtensa LX6 dual-core · 240 MHz · Wi-Fi + Bluetooth",
        tagline: "O ponto de partida ideal",
        desc: "A placa mais popular da família ESP32. Traz Wi-Fi, Bluetooth clássico e BLE, dois núcleos de até 240 MHz, muitos GPIOs e conversor USB-serial na própria placa. Com tutoriais e bibliotecas para quase tudo, é a melhor escolha para começar.",
        tip: "Se a placa não aparecer no computador, instale o driver do conversor USB-serial (CP210x ou CH340, conforme o modelo) e confirme que o cabo transmite dados, não só carga.",
        uses: &[
            "Automação residencial e IoT",
            "Servidores web e painéis de controle",
            "Laboratório de redes Wi-Fi e BLE",
        ],
    },
    Item {
        name: "ESP32-C3 DevKit",
        cat: Cat::Esp32,
        icon: board(1.9, 12, true, true, 1),
        level: Ini,
        chip: "ESP32-C3 · RISC-V 160 MHz · Wi-Fi + Bluetooth 5 (LE) · USB nativo",
        tagline: "RISC-V barato com USB nativo",
        desc: "Um núcleo RISC-V de até 160 MHz com Wi-Fi e Bluetooth 5 LE, consumo menor e custo mais baixo que o ESP32 clássico. O USB nativo permite gravar e depurar só com um cabo, sem conversor externo.",
        tip: "O C3 tem apenas BLE, sem Bluetooth clássico. Projetos que dependem de áudio ou perfil serial Bluetooth clássico precisam de um ESP32 original.",
        uses: &[
            "Sensores conectados de baixo custo",
            "Dispositivos BLE e beacons",
            "Projetos pequenos alimentados por bateria",
        ],
    },
    Item {
        name: "ESP32-C3 SuperMini",
        cat: Cat::Esp32,
        icon: board(1.3, 8, true, true, 1),
        level: Mid,
        chip: "ESP32-C3 · RISC-V · USB-C · cerca de 22 x 18 mm",
        tagline: "Do tamanho de um selo",
        desc: "Um ESP32-C3 com cerca de 22 x 18 mm, USB-C e antena cerâmica na própria placa. Perfeito onde o espaço é curto. O desenho da antena varia entre fabricantes, então vale testar o alcance do Wi-Fi no seu lote.",
        tip: "Se o Wi-Fi ficar instável, reduzir a potência de transmissão no código (setTxPower) costuma ajudar. Mantenha a antena longe de metais e da bateria.",
        uses: &[
            "Wearables e gadgets discretos",
            "Controles remotos Wi-Fi/BLE",
            "Sensores escondidos em caixas pequenas",
        ],
    },
    Item {
        name: "ESP32-S3 DevKitC",
        cat: Cat::Esp32,
        icon: board(2.2, 18, true, true, 1),
        level: Mid,
        chip: "ESP32-S3 · Xtensa LX7 dual-core · 240 MHz · Bluetooth 5 (LE) · USB OTG",
        tagline: "Potência, USB OTG e IA",
        desc: "Dois núcleos LX7 de até 240 MHz, Wi-Fi, Bluetooth 5 LE, USB OTG nativo e instruções de vetor que aceleram IA na borda. É a opção mais completa quando o projeto pede USB, câmera ou processamento pesado.",
        tip: "Muitas versões têm duas portas USB-C: uma para o conversor serial (gravação e log) e outra para o USB nativo. Teste as duas se o computador não reconhecer a placa.",
        uses: &[
            "Dispositivos USB HID (teclado, mouse, armazenamento)",
            "Câmeras e visão computacional leve",
            "Reconhecimento de voz e IA embarcada",
        ],
    },
    Item {
        name: "XIAO ESP32-C3",
        cat: Cat::Esp32,
        icon: board(1.25, 7, true, false, 1),
        level: Mid,
        chip: "Seeed Studio · ESP32-C3 · 21 x 17,8 mm · carregador LiPo",
        tagline: "Minúscula e pronta p/ bateria",
        desc: "Placa ultracompacta da Seeed Studio, com 21 x 17,8 mm, USB-C, carregador de bateria LiPo integrado e conector para antena externa. Cabe onde quase nada cabe e já nasce pronta para rodar na bateria.",
        tip: "Para ganhar alcance de Wi-Fi, conecte uma antena externa ao conector U.FL antes de ligar a placa.",
        uses: &[
            "Dispositivos vestíveis",
            "Sensores portáteis a bateria",
            "Protótipos que precisam de antena externa",
        ],
    },
    Item {
        name: "XIAO ESP32-S3",
        cat: Cat::Esp32,
        icon: board(1.25, 7, true, false, 1),
        level: Mid,
        chip: "Seeed Studio · ESP32-S3 · 8 MB PSRAM · 8 MB flash · carregador LiPo",
        tagline: "S3 com PSRAM em um dedal",
        desc: "O XIAO com a força do ESP32-S3: dois núcleos, 8 MB de PSRAM e 8 MB de flash, com suporte a bateria LiPo. A versão Sense acrescenta câmera e microfone digital, ideal para visão e áudio em espaço mínimo.",
        tip: "Ative a opção de PSRAM (OPI PSRAM) nas configurações da placa na IDE. Sem isso, câmeras e buffers grandes falham por falta de memória.",
        uses: &[
            "Câmeras compactas (versão Sense)",
            "Assistentes de voz e áudio",
            "Projetos de IA na borda com pouco espaço",
        ],
    },
    Item {
        name: "WiFi LoRa 32 V3 (ESP32-S3)",
        cat: Cat::Esp32,
        icon: Icon::Oled,
        level: Adv,
        chip: "ESP32-S3 · LoRa SX1262 · OLED 0,96\" · Wi-Fi + BLE · USB-C",
        tagline: "LoRa + OLED + bateria",
        desc: "ESP32-S3 com rádio LoRa SX1262 de longo alcance, display OLED de 0,96\" e conector de bateria numa única placa. É a base para redes de sensores que funcionam longe de qualquer Wi-Fi.",
        tip: "Conecte sempre a antena antes de transmitir, para não danificar o rádio, e use só a faixa de frequência permitida na sua região (no Brasil costuma ser a de 915 MHz).",
        uses: &[
            "Redes de sensores sem internet",
            "Telemetria de longo alcance",
            "Comunicação em campo aberto e áreas remotas",
        ],
    },
    // ── Arduino ──
    Item {
        name: "Arduino Uno R3",
        cat: Cat::Arduino,
        icon: board(1.35, 14, true, false, 1),
        level: Ini,
        chip: "ATmega328P · 16 MHz · 5 V · 14 digitais + 6 analógicos",
        tagline: "O clássico para aprender",
        desc: "Simples, robusto e compatível com milhares de shields, bibliotecas e tutoriais. Roda a 16 MHz com lógica de 5 V, o que facilita ligar a maioria dos sensores e módulos. É a escolha ideal para os primeiros passos em eletrônica.",
        tip: "Os pinos 0 e 1 são compartilhados com a USB: desconecte o que estiver ligado neles antes de gravar o programa.",
        uses: &[
            "Aprender eletrônica e lógica de programação",
            "Robôs e carrinhos simples",
            "Prototipagem com shields",
        ],
    },
    Item {
        name: "Arduino Uno R4 WiFi",
        cat: Cat::Arduino,
        icon: board(1.35, 14, true, false, 2),
        level: Ini,
        chip: "Renesas RA4M1 · Cortex-M4 48 MHz · 5 V · ESP32-S3 para Wi-Fi/BLE",
        tagline: "Uno moderno com Wi-Fi",
        desc: "Evolução do Uno com processador ARM Cortex-M4 de 48 MHz, matriz de LEDs 12x8 e Wi-Fi/Bluetooth feitos por um ESP32-S3. Mantém o formato e os 5 V do Uno, então os shields antigos continuam servindo.",
        tip: "Tem conector Qwiic para ligar sensores I2C sem solda e aceita tensões maiores no pino VIN que o Uno R3, o que ajuda a alimentar motores e fitas de LED.",
        uses: &[
            "Projetos educacionais conectados",
            "Painéis com a matriz de LEDs integrada",
            "Migrar projetos do Uno para IoT",
        ],
    },
    Item {
        name: "Arduino Nano",
        cat: Cat::Arduino,
        icon: board(2.6, 15, true, false, 1),
        level: Ini,
        chip: "ATmega328 · 16 MHz · 5 V · 18 x 45 mm",
        tagline: "Uno em miniatura",
        desc: "O mesmo cérebro do Uno (ATmega328) em formato de protoboard, com USB na própria placa. Pequeno, barato e muito versátil para quem precisa economizar espaço.",
        tip: "Muitos clones usam o conversor CH340: se o computador não reconhecer a placa, instale o driver. Se a gravação falhar, tente a opção “Old Bootloader” na IDE.",
        uses: &[
            "Projetos compactos em protoboard",
            "Controladores embutidos em caixas",
            "Sensores e automação simples",
        ],
    },
    Item {
        name: "Arduino Mega 2560",
        cat: Cat::Arduino,
        icon: board(2.1, 26, true, false, 1),
        level: Mid,
        chip: "ATmega2560 · 54 I/O digitais · 16 analógicos · 4 UARTs · 256 KB flash",
        tagline: "Pinos de sobra para tudo",
        desc: "Feita para projetos grandes: 54 pinos digitais, 16 analógicos, 256 KB de flash e quatro portas seriais para conversar com vários módulos ao mesmo tempo.",
        tip: "Os pinos de SPI (50 a 53) e I2C (20 e 21) ficam em lugares diferentes dos do Uno. Confira antes de encaixar shields ou ligar módulos.",
        uses: &[
            "Impressoras 3D e máquinas CNC",
            "Robôs com muitos sensores e motores",
            "Painéis e automação industrial didática",
        ],
    },
    Item {
        name: "Arduino Leonardo",
        cat: Cat::Arduino,
        icon: board(1.35, 14, true, false, 1),
        level: Mid,
        chip: "ATmega32U4 · 16 MHz · 5 V · USB nativo",
        tagline: "USB nativo: vira teclado",
        desc: "O USB nativo (ATmega32U4) permite que a placa seja reconhecida como teclado, mouse ou gamepad (HID). É muito usada em automação de tarefas e em estudos de segurança HID feitos em laboratório próprio e com autorização.",
        tip: "Como a USB é do próprio chip, um erro no código pode fazer a porta sumir. Para regravar, toque duas vezes no botão reset em sequência rápida e envie o programa.",
        uses: &[
            "Teclados e macro pads personalizados",
            "Controles e periféricos USB caseiros",
            "Demonstrações de segurança HID em ambiente controlado",
        ],
    },
    Item {
        name: "Arduino Micro",
        cat: Cat::Arduino,
        icon: board(2.6, 12, true, false, 1),
        level: Mid,
        chip: "ATmega32U4 · 16 MHz · 5 V · USB nativo · 48 x 18 mm",
        tagline: "Leonardo para a protoboard",
        desc: "Um Leonardo em tamanho mini (48 x 18 mm), com USB nativo e formato que encaixa direto na protoboard. Ideal para teclados e controles que precisam ser pequenos.",
        tip: "Usa o mesmo chip do Leonardo, então as bibliotecas de teclado e mouse USB funcionam sem mudanças. Só precisa escolher a placa correta na IDE.",
        uses: &[
            "Teclados mecânicos customizados",
            "Controles de jogo e de mídia",
            "Dispositivos USB compactos",
        ],
    },
    Item {
        name: "Arduino Pro Mini",
        cat: Cat::Arduino,
        icon: board(1.5, 12, false, false, 1),
        level: Mid,
        chip: "ATmega328P · 3,3 V/8 MHz ou 5 V/16 MHz · sem USB",
        tagline: "Leve e econômico, sem USB",
        desc: "Sem porta USB, é gravada por um adaptador USB-serial (FTDI). Sem conversor embutido, consome pouco e é ótima para projetos de baixo consumo e produtos finais pequenos.",
        tip: "Confira se a placa é de 3,3 V/8 MHz ou 5 V/16 MHz antes de alimentar e de escolher a opção na IDE. Ligar a tensão errada pode danificar a placa e os sensores.",
        uses: &[
            "Sensores a bateria de longa duração",
            "Produtos finais pequenos e baratos",
            "Projetos embarcados de baixo consumo",
        ],
    },
    Item {
        name: "Arduino Due",
        cat: Cat::Arduino,
        icon: board(2.0, 22, true, false, 1),
        level: Adv,
        chip: "AT91SAM3X8E · Cortex-M3 84 MHz · 3,3 V · 54 I/O · 2 DACs",
        tagline: "ARM de 32 bits e 2 DACs",
        desc: "O primeiro Arduino com ARM de 32 bits: Cortex-M3 de 84 MHz, 512 KB de flash, 54 pinos e dois conversores DAC. Entrega bem mais desempenho que as placas AVR, mas opera em 3,3 V.",
        tip: "Os pinos NÃO toleram 5 V. Ligar um módulo de 5 V direto neles pode danificar a placa; use um conversor de nível.",
        uses: &[
            "Processamento de áudio e sinais",
            "Aplicações que precisam de mais desempenho",
            "Geração de formas de onda com DAC",
        ],
    },
    Item {
        name: "Arduino Nano Every",
        cat: Cat::Arduino,
        icon: board(2.6, 15, true, false, 1),
        level: Ini,
        chip: "ATmega4809 · 20 MHz · 5 V · 48 KB flash · 6 KB SRAM",
        tagline: "Nano com mais memória",
        desc: "Um Nano modernizado, com ATmega4809 de 20 MHz, mais memória e mais periféricos que o Nano clássico, no mesmo formato e com a mesma pinagem.",
        tip: "O chip é diferente do Nano clássico: códigos que mexem direto nos registradores do ATmega328 não funcionam. Bibliotecas comuns costumam rodar normalmente.",
        uses: &[
            "Atualizar projetos feitos com o Nano",
            "Dispositivos pequenos com mais memória",
            "Controladores embutidos baratos",
        ],
    },
    // ── Displays ──
    Item {
        name: "OLED 0.96\" I2C",
        cat: Cat::Display,
        icon: Icon::Oled,
        level: Ini,
        chip: "SSD1306 · 128x64 · I2C · 3,3 V a 5 V",
        tagline: "Tela nítida com 4 fios",
        desc: "Tela monocromática de alto contraste que usa só 4 fios (VCC, GND, SDA e SCL). Perfeita para status, menus e gráficos, tanto no ESP32 quanto no Arduino.",
        tip: "O endereço I2C mais comum é 0x3C (alguns módulos usam 0x3D). Se a tela ficar em branco, rode um I2C scanner antes de suspeitar da biblioteca.",
        uses: &[
            "Menus e mensagens de status",
            "Gráficos e leituras de sensores",
            "Mini painéis em dispositivos portáteis",
        ],
    },
    Item {
        name: "OLED 1.3\" I2C",
        cat: Cat::Display,
        icon: Icon::Oled,
        level: Ini,
        chip: "SH1106 · 128x64 · I2C · 3,3 V a 5 V",
        tagline: "Maior e mais legível",
        desc: "Tela maior com a mesma resolução (128x64) do modelo de 0,96\", ótima para letras grandes e leitura à distância. Muitos módulos de 1,3\" usam o controlador SH1106, que não é igual ao SSD1306.",
        tip: "Se a imagem sair deslocada ou com uma faixa de ruído na lateral, troque a biblioteca para o driver do SH1106.",
        uses: &[
            "Interfaces com letras maiores e mais legíveis",
            "Relógios, medidores e dashboards",
            "Projetos que ficam à vista do usuário",
        ],
    },
    Item {
        name: "LCD 16x2 I2C",
        cat: Cat::Display,
        icon: Icon::Lcd { rows: 2 },
        level: Ini,
        chip: "HD44780 · 16 colunas x 2 linhas · I2C · backlight",
        tagline: "Texto simples e legível",
        desc: "Display de caracteres com módulo I2C que reduz a fiação a 4 fios. Simples de usar e legível até sob luz forte.",
        tip: "Se só aparecerem quadradinhos ou nada, ajuste o trimpot de contraste atrás da placa e confirme o endereço I2C (geralmente 0x27 ou 0x3F).",
        uses: &[
            "Termômetros, relógios e contadores",
            "Mensagens de status curtas",
            "Interfaces de texto para iniciantes",
        ],
    },
    Item {
        name: "LCD 20x4 I2C",
        cat: Cat::Display,
        icon: Icon::Lcd { rows: 4 },
        level: Ini,
        chip: "HD44780 · 20 colunas x 4 linhas · I2C · backlight",
        tagline: "Quatro linhas de informação",
        desc: "Mais linhas e colunas para mostrar menus, logs e várias leituras ao mesmo tempo. Usa o mesmo controlador do 16x2, então o código quase não muda: basta informar 20 colunas e 4 linhas.",
        tip: "O backlight consome bastante corrente. Em projetos a bateria, desligue-o quando não precisar da tela.",
        uses: &[
            "Menus com várias opções",
            "Monitor de vários sensores ao mesmo tempo",
            "Logs simples na bancada",
        ],
    },
    Item {
        name: "Display IPS 1.28 Redondo GC9A01",
        cat: Cat::Display,
        icon: Icon::Perf,
        level: Mid,
        chip: "GC9A01 · 240 x 240 · IPS colorido · SPI · 3,3 V",
        tagline: "Tela redonda e colorida",
        desc: "Display IPS redondo de 1,28 polegada com 240 x 240 pixels e controlador GC9A01, ligado por SPI. Ótimo para relógios, medidores e olhos animados, e funciona bem com ESP32 e bibliotecas como TFT_eSPI e LovyanGFX.",
        tip: "Alimente com 3,3 V e confira os pinos RST, CS, DC, SDA e SCL no código. Cores trocadas (azul no lugar de vermelho) costumam ser só a inversão de ordem RGB/BGR na biblioteca.",
        uses: &[
            "Relógios e medidores circulares",
            "Olhos animados e mascotes",
            "Painéis compactos de status",
        ],
    },
    // ── Módulos ──
    Item {
        name: "Leitor NFC PN532",
        cat: Cat::Modulo,
        icon: Icon::Nfc,
        level: Mid,
        chip: "PN532 · 13,56 MHz · I2C / SPI / UART · ISO14443 A/B",
        tagline: "NFC/RFID completo",
        desc: "Lê e grava tags NFC/RFID e suporta modos avançados, como comunicação peer-to-peer. É muito usado em estudos de controle de acesso e de segurança de cartões, sempre em equipamentos próprios ou com autorização.",
        tip: "Confira as chaves DIP da placa: elas escolhem entre I2C, SPI e UART, e a interface errada é a causa mais comum de “módulo não encontrado”.",
        uses: &[
            "Leitura e gravação de tags NDEF",
            "Controle de acesso didático",
            "Estudo de segurança de cartões em laboratório",
        ],
    },
    Item {
        name: "Leitor NFC RC522",
        cat: Cat::Modulo,
        icon: Icon::Nfc,
        level: Ini,
        chip: "MFRC522 · 13,56 MHz · SPI · 3,3 V",
        tagline: "RFID barato para começar",
        desc: "Módulo econômico para ler cartões e chaveiros Mifare. O alcance é curto (poucos centímetros) e existe muito material de apoio, o que o torna ótimo para os primeiros projetos com RFID.",
        tip: "Alimente com 3,3 V, nunca com 5 V. Mantenha a tag parada a poucos centímetros da antena durante a leitura.",
        uses: &[
            "Fechaduras e portas com cartão",
            "Registro de presença",
            "Primeiros projetos com RFID",
        ],
    },
    Item {
        name: "CYD ESP32 Touch",
        cat: Cat::Modulo,
        icon: Icon::Cyd,
        level: Mid,
        chip: "ESP32-2432S028R · TFT 2,8\" 320x240 · touch resistivo · microSD",
        tagline: "ESP32 com tela touch",
        desc: "Cheap Yellow Display: um ESP32 com tela touch colorida de 2,8\", slot microSD, LED RGB e sensor de luz numa placa só. Queridinho da comunidade para dashboards e ferramentas portáteis.",
        tip: "Existem lotes com controladores de tela diferentes. Se a tela ficar em branco ou com cores invertidas, ajuste o driver (ILI9341 ou ST7789) e a inversão de cores.",
        uses: &[
            "Dashboards touch de IoT",
            "Ferramentas portáteis com interface gráfica",
            "Projetos com LVGL e TFT_eSPI",
        ],
    },
    Item {
        name: "Teclado Membrana 4x3",
        cat: Cat::Modulo,
        icon: Icon::Pcb,
        level: Ini,
        chip: "Matriz 4x3 · 12 teclas · 7 pinos · autoadesivo",
        tagline: "12 teclas autoadesivas",
        desc: "Teclado matricial flexível e autoadesivo com 12 teclas (0 a 9, * e #). Ocupa pouco espaço e usa varredura de linhas e colunas, que a biblioteca Keypad resolve em poucas linhas de código.",
        tip: "Os 7 pinos são 4 linhas e 3 colunas, mas a ordem varia entre fabricantes. Teste com um programa que imprima a tecla pressionada e ajuste o mapa de pinos.",
        uses: &[
            "Senhas e travas eletrônicas",
            "Menus numéricos",
            "Entrada de dados em projetos com ESP32 ou Arduino",
        ],
    },
    Item {
        name: "Módulo Micro SD (TF Card)",
        cat: Cat::Modulo,
        icon: Icon::Pcb,
        level: Ini,
        chip: "Interface SPI · microSD / TF · FAT32",
        tagline: "Armazenamento para seus dados",
        desc: "Adaptador para ler e gravar cartões microSD via SPI. Use cartões formatados em FAT32 e confirme a tensão do módulo: muitos exigem 5 V na alimentação, mas os sinais do ESP32 são de 3,3 V.",
        tip: "Cartões menores (até 32 GB) em FAT32 dão menos problema. Se a inicialização falhar, teste outro cartão e confira os fios SPI antes de mexer no código.",
        uses: &[
            "Registro (log) de dados de sensores",
            "Armazenar capturas e arquivos",
            "Tocar áudio e carregar imagens",
        ],
    },
    Item {
        name: "Sensor de Chuva",
        cat: Cat::Modulo,
        icon: Icon::Pcb,
        level: Ini,
        chip: "Placa de gotas + módulo LM393 · saída analógica e digital",
        tagline: "Detecta gotas de água",
        desc: "Detecta gotas de água na placa de trilhas e entrega o sinal pelo módulo comparador LM393, com saída analógica e digital e sensibilidade ajustável no trimpot.",
        tip: "A placa sofre corrosão com o uso contínuo. Alimente-a por um pino digital, só no momento da leitura, e ela vai durar muito mais.",
        uses: &[
            "Estações meteorológicas",
            "Fechar janelas ou toldos automaticamente",
            "Alertas de chuva em projetos de irrigação",
        ],
    },
    Item {
        name: "Módulo nRF24L01 com Antena",
        cat: Cat::Modulo,
        icon: Icon::Pcb,
        level: Mid,
        chip: "nRF24L01 + PA/LNA · 2,4 GHz · SPI · antena SMA · 3,3 V",
        tagline: "Rádio 2,4 GHz de longo alcance",
        desc: "Rádio de 2,4 GHz para comunicação direta entre placas, com amplificador (PA/LNA) e antena externa para ganhar alcance. Consome picos de corrente, então a alimentação precisa ser estável.",
        tip: "Coloque um capacitor entre VCC e GND, bem perto do módulo (algo entre 10 µF e 100 µF), e use 3,3 V estável. Sem isso, a comunicação costuma falhar de forma intermitente.",
        uses: &[
            "Controles remotos e drones simples",
            "Redes de sensores locais",
            "Comunicação entre duas ou mais placas",
        ],
    },
    Item {
        name: "Microfone INMP441 I2S",
        cat: Cat::Modulo,
        icon: Icon::Perf,
        level: Mid,
        chip: "INMP441 · microfone MEMS digital · I2S · 3,3 V",
        tagline: "Áudio digital de alta qualidade",
        desc: "Microfone MEMS omnidirecional com saída digital I2S, sem ruído analógico no caminho. O ESP32 tem I2S nativo, então a captura de áudio é direta, com pinos SCK, WS, SD, L/R, VDD e GND.",
        tip: "O pino L/R escolhe o canal: ligado ao GND é o canal esquerdo, ao VDD é o direito. Se não vier áudio, confira se o canal configurado no código é o mesmo.",
        uses: &[
            "Gravação e streaming de áudio",
            "Detecção de som e palmas",
            "Reconhecimento de voz em ESP32",
        ],
    },
    Item {
        name: "Microfone MAX4466",
        cat: Cat::Modulo,
        icon: Icon::Perf,
        level: Ini,
        chip: "MAX4466 · eletreto + amplificador · saída analógica · 2,4 a 5,5 V",
        tagline: "Microfone com ganho ajustável",
        desc: "Microfone de eletreto com amplificador MAX4466 e trimpot de ganho. A saída é analógica (OUT), lida por qualquer entrada ADC, e o ganho se ajusta para captar de sussurros a ambientes barulhentos.",
        tip: "No ESP32 use um pino ADC1, porque o ADC2 conflita com o Wi-Fi. Comece com o ganho baixo para evitar saturar o sinal.",
        uses: &[
            "Medidor de nível sonoro",
            "Acionar algo com palmas",
            "Visualizadores de áudio",
        ],
    },
    Item {
        name: "Sensor de Toque TTP223",
        cat: Cat::Modulo,
        icon: Icon::Perf,
        level: Ini,
        chip: "TTP223 · toque capacitivo · 2 a 5,5 V · saída digital",
        tagline: "Botão sem partes móveis",
        desc: "Módulo de toque capacitivo que funciona como um botão sem contato mecânico. Entrega nível alto ao toque e dá para escolher entre modo momentâneo ou alternado por jumpers de solda na placa.",
        tip: "Evite encostar metais ou cabos longos na área de toque. A sensibilidade muda com a capa, e plástico fino por cima costuma funcionar bem.",
        uses: &[
            "Interruptores e botões discretos",
            "Acionar luzes e relés",
            "Interfaces com toque em caixas fechadas",
        ],
    },
    Item {
        name: "Sensor Ultrassônico HC-SR04",
        cat: Cat::Modulo,
        icon: Icon::Perf,
        level: Ini,
        chip: "HC-SR04 · 2 cm a 4 m · 5 V · Trig / Echo",
        tagline: "Mede distância com som",
        desc: "Emite um pulso ultrassônico no pino Trig e mede o tempo até o eco voltar pelo pino Echo, calculando a distância. É barato e preciso o bastante para robôs, níveis de líquido e alarmes de proximidade.",
        tip: "O módulo trabalha em 5 V e o Echo também sai em 5 V. Em um ESP32 use um divisor de tensão no Echo para proteger o pino de 3,3 V.",
        uses: &[
            "Robôs que desviam de obstáculos",
            "Medir nível de água",
            "Alarmes de presença",
        ],
    },
    Item {
        name: "Joystick Analógico 5V",
        cat: Cat::Modulo,
        icon: Icon::Perf,
        level: Ini,
        chip: "2 eixos analógicos (X/Y) + botão · 5 V",
        tagline: "Controle em dois eixos",
        desc: "Módulo com dois potenciômetros (eixos X e Y) e um botão no clique do direcional. As posições são lidas por entradas analógicas e o botão por uma entrada digital com pull-up.",
        tip: "Na posição central o valor fica perto da metade da escala. Faça uma calibração inicial e use uma zona morta para evitar movimentos fantasmas.",
        uses: &[
            "Controle de robôs e carrinhos",
            "Menus e jogos simples",
            "Câmeras com pan/tilt",
        ],
    },
    Item {
        name: "Receptor IR VS1838",
        cat: Cat::Modulo,
        icon: Icon::Perf,
        level: Ini,
        chip: "VS1838 / TL1838 · infravermelho 38 kHz · 2,7 a 5,5 V",
        tagline: "Recebe sinais de controle remoto",
        desc: "Receptor infravermelho de 38 kHz com filtro e demodulador integrados. Entrega o sinal já limpo no pino de dados, que as bibliotecas IRremote decodificam para ler botões de controles remotos comuns.",
        tip: "Os três pinos são sinal, GND e VCC, mas a ordem varia entre fabricantes. Confira o datasheet do seu lote antes de ligar para não inverter a alimentação.",
        uses: &[
            "Ler e clonar códigos de controle remoto",
            "Controlar projetos com controle de TV",
            "Estudo de protocolos IR como NEC",
        ],
    },
    Item {
        name: "Kit MAX98357 + Alto-falante 40mm",
        cat: Cat::Modulo,
        icon: Icon::Perf,
        level: Mid,
        chip: "MAX98357A · DAC + amplificador classe D I2S · 3 W · alto-falante 40 mm 4 ohms",
        tagline: "Som direto do ESP32",
        desc: "Amplificador classe D com DAC I2S integrado que recebe áudio digital do ESP32 e aciona direto um alto-falante de 4 ohms. Acompanha o alto-falante de 40 mm e 3 W, formando um conjunto completo de saída de áudio.",
        tip: "Os pinos são BCLK, LRC e DIN, e o pino GAIN define o ganho. Alimente com 5 V para ter mais volume e use fios curtos no alto-falante para reduzir ruído.",
        uses: &[
            "Tocar WAV e MP3 no ESP32",
            "Alertas e avisos sonoros",
            "Caixas de som e assistentes de voz",
        ],
    },
    // ── Energia ──
    Item {
        name: "Bateria LiPo 300 mAh",
        cat: Cat::Energia,
        icon: Icon::Battery { mah: 300 },
        level: Mid,
        chip: "3,7 V · LiPo · 300 mAh",
        tagline: "A menor da linha",
        desc: "Leve e discreta, feita para wearables e placas pequenas como XIAO e SuperMini. A autonomia é curta, então combine com modos de baixo consumo, como o deep sleep.",
        tip: "Confira a polaridade do conector (vermelho é o positivo) antes de plugar: conectores idênticos podem vir com os fios invertidos conforme o fabricante.",
        uses: &[
            "Wearables e crachás",
            "Placas ultracompactas",
            "Dispositivos de uso rápido",
        ],
    },
    Item {
        name: "Bateria LiPo 400 mAh",
        cat: Cat::Energia,
        icon: Icon::Battery { mah: 400 },
        level: Mid,
        chip: "3,7 V · LiPo · 400 mAh",
        tagline: "Compacta para deep sleep",
        desc: "Boa para projetos compactos de baixo consumo, que dormem a maior parte do tempo (deep sleep) e acordam só para medir e enviar dados.",
        tip: "Use sempre um módulo de carga com proteção, que evita sobrecarga e descarga profunda. Carregar uma LiPo sem proteção é arriscado.",
        uses: &[
            "Sensores com deep sleep",
            "Gadgets pequenos",
            "Reserva de energia para relógios e logs",
        ],
    },
    Item {
        name: "Bateria LiPo 500 mAh",
        cat: Cat::Energia,
        icon: Icon::Battery { mah: 500 },
        level: Mid,
        chip: "3,7 V · LiPo · 500 mAh",
        tagline: "Equilíbrio tamanho/autonomia",
        desc: "Equilíbrio entre tamanho e autonomia para sensores e gadgets portáteis que transmitem dados de tempos em tempos.",
        tip: "Prefira carregar com corrente baixa (cerca de metade da capacidade ou menos). A carga mais lenta preserva a vida útil da bateria.",
        uses: &[
            "Sensores que enviam dados periodicamente",
            "Gadgets portáteis de mão",
            "Protótipos de produtos pequenos",
        ],
    },
    Item {
        name: "Bateria LiPo 600 mAh",
        cat: Cat::Energia,
        icon: Icon::Battery { mah: 600 },
        level: Mid,
        chip: "3,7 V · LiPo · 600 mAh",
        tagline: "Fôlego para tela OLED",
        desc: "Mais autonomia para um ESP32 com display OLED ou com Wi-Fi ligado de vez em quando, sem ocupar muito espaço dentro da caixa.",
        tip: "Nunca descarregue abaixo de cerca de 3,0 V. O ideal é desligar o projeto perto de 3,3 V para preservar a bateria.",
        uses: &[
            "ESP32 com tela OLED",
            "Dispositivos com Wi-Fi ocasional",
            "Projetos portáteis do dia a dia",
        ],
    },
    Item {
        name: "Bateria LiPo 700 mAh",
        cat: Cat::Energia,
        icon: Icon::Battery { mah: 700 },
        level: Mid,
        chip: "3,7 V · LiPo · 700 mAh",
        tagline: "Para uso prolongado",
        desc: "Capacidade intermediária para projetos portáteis que ficam mais tempo ligados, como gravadores de dados e ferramentas de bancada.",
        tip: "Carregue só com carregador próprio para LiPo, longe de materiais inflamáveis, e nunca a deixe carregando sem supervisão.",
        uses: &[
            "Ferramentas portáteis de bancada",
            "Gravadores de dados (data loggers)",
            "Dispositivos de uso prolongado",
        ],
    },
    Item {
        name: "Bateria LiPo 800 mAh",
        cat: Cat::Energia,
        icon: Icon::Battery { mah: 800 },
        level: Mid,
        chip: "3,7 V · LiPo · 800 mAh",
        tagline: "A de maior autonomia",
        desc: "A mais robusta desta lista: ideal para ferramentas portáteis com tela e Wi-Fi ativos por mais tempo, como o CYD funcionando longe da tomada.",
        tip: "Evite calor, perfurações e curto-circuito. Se a bateria inchar, pare de usá-la e descarte em um ponto de coleta apropriado.",
        uses: &[
            "Ferramentas portáteis com tela e Wi-Fi",
            "CYD e displays touch a bateria",
            "Projetos de maior autonomia",
        ],
    },
    // ── PCB ──
    Item {
        name: "PCB Perfurada",
        cat: Cat::Pcb,
        icon: Icon::Perf,
        level: Ini,
        chip: "Perfboard · furos de 2,54 mm",
        tagline: "Do protótipo ao definitivo",
        desc: "Placa com furos no padrão de 2,54 mm para soldar circuitos à mão. É o passo natural entre o protótipo na protoboard e o projeto definitivo.",
        tip: "Planeje o layout no papel antes de soldar e mantenha as ligações de alimentação curtas e grossas. Isso evita retrabalho e mau contato.",
        uses: &[
            "Tornar um protótipo permanente",
            "Circuitos pequenos feitos à mão",
            "Treino de solda",
        ],
    },
    Item {
        name: "PCB Customizada",
        cat: Cat::Pcb,
        icon: Icon::Pcb,
        level: Adv,
        chip: "Projeto em KiCad · fabricação sob encomenda",
        tagline: "Acabamento de produto",
        desc: "Placa feita sob medida, com trilhas, vias e serigrafia próprias. Ideal para transformar seu protótipo em um produto de verdade, com acabamento profissional.",
        tip: "Antes de encomendar, rode a verificação de regras de projeto (DRC) no KiCad e confira os furos e as dimensões com uma impressão em escala 1:1.",
        uses: &[
            "Produtos finais e pequenas séries",
            "Circuitos compactos e confiáveis",
            "Projetos de portfólio",
        ],
    },
    // ── Componentes ──
    Item {
        name: "Kit Resistor 10k 1/4W",
        cat: Cat::Comp,
        icon: Icon::Perf,
        level: Ini,
        chip: "10 kΩ · 1/4 W · 5% · carbono · 200 unidades",
        tagline: "O valor clássico do pull-up",
        desc: "Kit com 200 resistores de carbono de 10 kΩ, o valor clássico para pull-up e pull-down em botões e entradas digitais. Garante que a entrada nunca fique \"flutuando\".",
        tip: "As faixas de um resistor de 10 kΩ são marrom, preto e laranja. Na dúvida, confirme com o multímetro antes de soldar.",
        uses: &[
            "Pull-up e pull-down de botões",
            "Divisores de tensão",
            "Ligação de sensores analógicos",
        ],
    },
    Item {
        name: "Jumpers Rígidos Coloridos",
        cat: Cat::Comp,
        icon: Icon::Perf,
        level: Ini,
        chip: "Kit com caixa organizadora · para protoboard",
        tagline: "Montagem limpa e organizada",
        desc: "Fios rígidos pré-cortados em vários comprimentos e cores, já dobrados para a protoboard. Deixam a montagem limpa e fácil de revisar.",
        tip: "Padronize as cores (por exemplo, vermelho para alimentação e preto para GND). Assim você revisa o circuito num relance.",
        uses: &[
            "Montagens organizadas em protoboard",
            "Conexões curtas e firmes",
            "Revisar circuitos com facilidade",
        ],
    },
    Item {
        name: "Protoboard 400 Pontos",
        cat: Cat::Comp,
        icon: Icon::Perf,
        level: Ini,
        chip: "400 furos · 8,5 x 5,5 cm",
        tagline: "Compacta para testes rápidos",
        desc: "Placa de ensaio compacta para montar circuitos sem solda. Boa para testes rápidos com ESP32, sensores e módulos pequenos.",
        tip: "Contato frouxo é a causa número um de circuito que para do nada. Evite forçar terminais muito grossos nos furos e troque os jumpers dobrados.",
        uses: &[
            "Testes rápidos de circuito",
            "Experimentos com um ou dois módulos",
            "Aprender eletrônica sem solda",
        ],
    },
    Item {
        name: "Protoboard 830 Pontos",
        cat: Cat::Comp,
        icon: Icon::Perf,
        level: Ini,
        chip: "830 furos · trilhas de alimentação duplas",
        tagline: "Espaço para projetos maiores",
        desc: "Protoboard grande para circuitos maiores, com trilhas de alimentação nos dois lados. Cabe uma placa de desenvolvimento e vários módulos ao mesmo tempo.",
        tip: "Em muitas protoboards desse tamanho as trilhas de alimentação são divididas ao meio. Se faltar tensão numa ponta, ligue as duas metades com um jumper.",
        uses: &[
            "Projetos com vários módulos",
            "Circuitos com alimentação separada",
            "Protótipos antes da PCB final",
        ],
    },
    Item {
        name: "Buzzer",
        cat: Cat::Comp,
        icon: Icon::Perf,
        level: Ini,
        chip: "Buzzer ativo · 5 V · ~85 dB · 2 terminais",
        tagline: "Apito simples e barulhento",
        desc: "Emite um som de frequência fixa quando recebe tensão, sem precisar gerar tom no código. Ligado a um pino digital (ou a um transistor), serve para avisos, alarmes e confirmações de ação.",
        tip: "Se o som sair em tom diferente ou não toque, veja se é buzzer ativo (tom fixo) ou passivo (precisa de PWM). Para correntes maiores use um transistor entre o pino e o buzzer.",
        uses: &[
            "Alarmes e avisos sonoros",
            "Confirmação de botão ou leitura NFC",
            "Sinal de erro em projetos",
        ],
    },
];
