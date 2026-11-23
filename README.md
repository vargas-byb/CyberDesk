# CyberDesk

Catálogo animado de hardware para cibersegurança: ESP32, Arduino, displays, módulos NFC, CYD, baterias, PCBs e componentes.
Feito em Rust com [egui](https://github.com/emilk/egui) — visual escuro, rápido e sem dependências: o `.exe` é autocontido.

## O que o app faz

- **Entra e já fica logado.** Marque *Manter conectado neste computador* (vem marcado) e, ao fechar e abrir o app de novo, você continua logado. A sessão vale por 90 dias a partir do último uso (renova a cada abertura). Ao clicar em **Sair da conta**, a sessão salva é apagada. O último usuário também fica preenchido na tela de login.
- **Conta local e segura.** Usuário + senha, com foto de perfil opcional. Tudo fica só neste PC; a senha é guardada com hash Argon2 e a sessão salva usa um token aleatório (só o hash dele é guardado).
- **Catálogo com 48 itens**, cada um com resumo, descrição, especificações, nível de dificuldade, usos comuns, **dica de bancada** e **itens relacionados** (clique para pular entre eles).
- **Favoritos** por usuário (estrela no card ou tecla `F`).
- **Busca inteligente** (ignora acentos; procura em nome, chip, descrição e usos) + filtros por categoria + ordenação (padrão, A–Z, nível).
- **Painel de detalhes** ao lado do card, com navegação ◀ ▶ entre os itens, copiar ficha e favoritar.
- **Animações e interações**: abertura animada, fundo com partículas e luz que segue o mouse, login em duas colunas com trilhas de circuito animadas, animação de "acesso concedido", cards com parallax e brilho, faíscas ao favoritar, busca com sugestões digitadas, botão de voltar ao topo, avisos (toasts) com barra de tempo, contadores animados, medidor de força de senha, arrastar e soltar a foto de perfil.

- **Atualização dentro do app.** No canto inferior direito há um botão de atualização. A cada abertura (e a cada 30 min) o app consulta a última Release do GitHub; se houver versão nova, aparece uma **bolinha amarela** no botão e um aviso. Ao clicar em **ATUALIZAR**, o app baixa o novo `CyberDesk.exe` (conferindo tamanho e SHA-256), troca o executável e reabre sozinho. Seus dados em `%APPDATA%\CyberDesk` não são tocados.

## Atalhos

| Tecla | Ação |
|---|---|
| `/` ou `Ctrl+F` | Focar a busca |
| `←` `→` | Item anterior / próximo (com o painel aberto) |
| `F` | Favoritar / desfavoritar (com o painel aberto) |
| `ESC` | Fecha menu, painel ou limpa a busca |
| `Enter` (login) | Avança para o próximo campo; envia ao terminar |

## Onde ficam os dados

Em `%APPDATA%\CyberDesk` (Windows): `users.json` (contas), `session.json` (sessão salva), `favorites.json` e a pasta `avatars/`.
Para "esquecer" este computador, basta usar **Sair da conta** ou apagar `session.json`.

## Imagens dos itens

Coloque as imagens em `assets/items/` (nomes em `assets/items/LEIA-ME.md`). Elas são embutidas no `.exe`.
Item sem imagem usa um desenho animado feito em código.

## Ícone do app

`assets/icon/` guarda o ícone (`icon.png`, `icon-1024.png` e `cyberdesk.ico`). O `.ico` é embutido no `CyberDesk.exe` no build do Windows (`build.rs`) e o `icon.png` é usado na janela e na tela de abertura.

## Rodar localmente

```
cargo run --release
cargo test    # testes do login salvo
```

## Gerar o .exe pelo GitHub

Todo push na `main` compila o app e publica uma **Release** nova (`v0.4.<build>`) com o `CyberDesk.exe` anexado,
na aba **Releases** do repositório. O executável é autocontido: quem for usar **não precisa instalar Rust**.
O número do build (`<build>`) é gravado dentro do `.exe` na compilação (variável `CYBERDESK_BUILD`); é com ele que o app descobre se existe versão mais nova. Builds locais (`cargo run`) não têm esse número e não se atualizam sozinhos.

## Fontes

[Inter](https://github.com/rsms/inter) e [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono), ambas sob a licença SIL OFL 1.1.
