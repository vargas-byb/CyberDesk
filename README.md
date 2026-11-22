# CyberDesk

Catálogo animado de hardware para cibersegurança: ESP32, Arduino, displays, módulos NFC, CYD, antenas, baterias e PCBs.
Feito em Rust com [egui](https://github.com/emilk/egui), visual preto e branco.

## Como usar

- Clique em qualquer card para abrir o painel lateral com nome e descrição.
- `ESC` ou clique fora do painel para fechar.
- Use a busca e os filtros de categoria no topo.

## Rodar localmente

```
cargo run --release
```

## Gerar o .exe pelo GitHub

- Todo push na `main` compila o app; o `CyberDesk.exe` fica em **Actions → Build CyberDesk → Artifacts**.
- Para publicar uma Release com o `.exe`:

```
git tag v0.1.0
git push origin v0.1.0
```

O executável é autocontido: quem for usar **não precisa instalar Rust**.
