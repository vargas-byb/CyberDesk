# CyberDesk

Catálogo animado de hardware para cibersegurança: ESP32, Arduino, displays, módulos NFC, CYD, antenas, baterias e PCBs.
Feito em Rust com [egui](https://github.com/emilk/egui), visual preto e branco.

## Como usar

- Crie uma conta local (usuário + senha, com foto de perfil opcional) e entre. Os dados ficam só neste PC (senha com hash Argon2).
- Clique em um card: ele fica em destaque, o fundo escurece e o popup abre ao lado dele.
- `ESC` ou clique fora para fechar. Use a busca e os filtros no topo.
- Foto de perfil: clique no seu nome (canto superior direito) → TROCAR FOTO, ou arraste uma imagem para a janela.

## Imagens dos itens

Coloque as imagens em `assets/items/` (nomes em `assets/items/LEIA-ME.md`). Elas são embutidas no `.exe`.

## Rodar localmente

```
cargo run --release
```

## Gerar o .exe pelo GitHub

Todo push na `main` compila o app e publica uma **Release** nova (`v0.2.<build>`) com o `CyberDesk.exe` anexado,
na aba **Releases** do repositório. O executável é autocontido: quem for usar **não precisa instalar Rust**.
