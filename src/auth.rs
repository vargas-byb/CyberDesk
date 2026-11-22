//! Login local: usuários e senhas ficam só neste computador.
//! Senhas são guardadas como hash Argon2id (nunca em texto puro).
//!
//! Sessão salva: ao marcar "manter conectado", um token aleatório de 256 bits
//! é gravado em `session.json`; só o hash Argon2 dele fica em `users.json`.
//! Assim, ao abrir o app de novo, a pessoa continua logada sem digitar a senha.
//! A sessão vale por 90 dias a partir do último uso (renova a cada abertura).
//! Sair (logout) apaga o token.

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// Quantos dias a sessão salva continua válida sem o app ser aberto.
pub const SESSION_DAYS: u64 = 90;
const DAY: u64 = 86_400;

#[derive(Serialize, Deserialize, Clone)]
struct User {
    name: String,
    hash: String,
    /// Hash Argon2 do token da sessão salva (None = sem sessão salva).
    #[serde(default)]
    session: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct SessionFile {
    user: String,
    token: String,
    /// Momento (segundos desde 1970) em que a sessão expira. 0 = arquivo antigo,
    /// sem validade registrada: é aceito e já passa a ter prazo.
    #[serde(default)]
    expires: u64,
}

#[derive(Serialize, Deserialize, Default)]
struct Db {
    users: Vec<User>,
}

/// Preferências simples (não sensíveis).
#[derive(Serialize, Deserialize, Default)]
struct Prefs {
    #[serde(default)]
    last_user: Option<String>,
}

pub struct Store {
    dir: PathBuf,
    db: Db,
    /// Quando a sessão atual expira (só preenchido se há sessão salva ativa).
    session_expires: Option<u64>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn data_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_DATA_HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("CyberDesk")
}

/// Grava o arquivo de forma atômica (escreve num temporário e renomeia),
/// para que um fechamento brusco do app nunca deixe o arquivo pela metade.
fn write_atomic(path: &Path, contents: &str, private: bool) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, contents)?;
    #[cfg(unix)]
    if private {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = private;
    fs::rename(&tmp, path)
}

impl Store {
    pub fn load() -> Self {
        Self::open(data_dir())
    }

    /// Abre (ou cria) o armazenamento numa pasta específica.
    pub fn open(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(dir.join("avatars"));
        let file = dir.join("users.json");
        let db = match fs::read_to_string(&file) {
            Ok(s) => match serde_json::from_str::<Db>(&s) {
                Ok(d) => d,
                Err(_) => {
                    // arquivo corrompido: guarda cópia em vez de sobrescrever
                    let _ = fs::rename(&file, dir.join("users.json.bak"));
                    Db::default()
                }
            },
            Err(_) => Db::default(),
        };
        Self {
            dir,
            db,
            session_expires: None,
        }
    }

    fn save(&self) -> Result<(), String> {
        let s = serde_json::to_string_pretty(&self.db).map_err(|e| e.to_string())?;
        write_atomic(&self.dir.join("users.json"), &s, true)
            .map_err(|_| "Não foi possível salvar os dados".to_string())
    }

    pub fn register(&mut self, name: &str, pw: &str) -> Result<String, String> {
        let name = name.trim();
        if name.len() < 3 || name.len() > 20 {
            return Err("O usuário deve ter de 3 a 20 caracteres".into());
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return Err("Use só letras, números, _ - e .".into());
        }
        if pw.chars().count() < 6 {
            return Err("A senha deve ter pelo menos 6 caracteres".into());
        }
        if self
            .db
            .users
            .iter()
            .any(|u| u.name.eq_ignore_ascii_case(name))
        {
            return Err("Esse usuário já existe".into());
        }
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(pw.as_bytes(), &salt)
            .map_err(|_| "Falha ao proteger a senha".to_string())?
            .to_string();
        self.db.users.push(User {
            name: name.to_string(),
            hash,
            session: None,
        });
        if let Err(e) = self.save() {
            self.db.users.pop();
            return Err(e);
        }
        Ok(name.to_string())
    }

    pub fn login(&self, name: &str, pw: &str) -> Result<String, String> {
        let bad = || "Usuário ou senha inválidos".to_string();
        let u = self
            .db
            .users
            .iter()
            .find(|u| u.name.eq_ignore_ascii_case(name.trim()))
            .ok_or_else(bad)?;
        let parsed = PasswordHash::new(&u.hash).map_err(|_| bad())?;
        Argon2::default()
            .verify_password(pw.as_bytes(), &parsed)
            .map_err(|_| bad())?;
        Ok(u.name.clone())
    }

    // ───── Sessão salva ─────

    fn session_file(&self) -> PathBuf {
        self.dir.join("session.json")
    }

    /// Cria (ou apaga, se `remember` for falso) a sessão salva do usuário.
    pub fn remember(&mut self, name: &str, remember: bool) {
        if !remember {
            self.forget(name);
            return;
        }
        let mut raw = [0u8; 32];
        if getrandom_fill(&mut raw).is_err() {
            return;
        }
        let token: String = raw.iter().map(|b| format!("{b:02x}")).collect();
        let salt = SaltString::generate(&mut OsRng);
        let Ok(hash) = Argon2::default().hash_password(token.as_bytes(), &salt) else {
            return;
        };
        let hash = hash.to_string();
        let Some(u) = self
            .db
            .users
            .iter_mut()
            .find(|u| u.name.eq_ignore_ascii_case(name))
        else {
            return;
        };
        let expires = now_secs() + SESSION_DAYS * DAY;
        let file = SessionFile {
            user: u.name.clone(),
            token,
            expires,
        };
        let ok = serde_json::to_string(&file)
            .ok()
            .map(|j| write_atomic(&self.dir.join("session.json"), &j, true).is_ok())
            .unwrap_or(false);
        if ok {
            u.session = Some(hash);
            self.session_expires = Some(expires);
            let _ = self.save();
        }
    }

    /// Restaura a sessão salva, se existir e ainda for válida. Ao restaurar,
    /// o prazo é renovado (quem usa o app com frequência nunca precisa entrar de novo).
    pub fn restore_session(&mut self) -> Option<String> {
        let raw = fs::read_to_string(self.session_file()).ok()?;
        let file: SessionFile = serde_json::from_str(&raw).ok()?;
        if file.expires != 0 && file.expires < now_secs() {
            // sessão vencida: apaga o token e pede login de novo
            let _ = fs::remove_file(self.session_file());
            return None;
        }
        let name = {
            let u = self
                .db
                .users
                .iter()
                .find(|u| u.name.eq_ignore_ascii_case(&file.user))?;
            let parsed = PasswordHash::new(u.session.as_deref()?).ok()?;
            Argon2::default()
                .verify_password(file.token.as_bytes(), &parsed)
                .ok()?;
            u.name.clone()
        };
        // renova o prazo (sessão deslizante)
        let expires = now_secs() + SESSION_DAYS * DAY;
        let renewed = SessionFile {
            user: file.user,
            token: file.token,
            expires,
        };
        if let Ok(j) = serde_json::to_string(&renewed) {
            let _ = write_atomic(&self.session_file(), &j, true);
        }
        self.session_expires = Some(expires);
        Some(name)
    }

    /// Dias que faltam para a sessão salva vencer (None = não há sessão salva).
    pub fn session_days_left(&self) -> Option<u64> {
        self.session_expires
            .map(|e| e.saturating_sub(now_secs()).div_ceil(DAY))
    }

    /// Apaga a sessão salva (logout).
    pub fn forget(&mut self, name: &str) {
        let _ = fs::remove_file(self.session_file());
        self.session_expires = None;
        if let Some(u) = self
            .db
            .users
            .iter_mut()
            .find(|u| u.name.eq_ignore_ascii_case(name))
        {
            if u.session.take().is_some() {
                let _ = self.save();
            }
        }
    }

    // ───── Preferências ─────

    fn prefs_file(&self) -> PathBuf {
        self.dir.join("prefs.json")
    }

    /// Último usuário que entrou (para já deixar o campo preenchido no login).
    pub fn last_user(&self) -> Option<String> {
        let p: Prefs = fs::read_to_string(self.prefs_file())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        p.last_user
            .filter(|n| self.db.users.iter().any(|u| u.name == *n))
    }

    pub fn set_last_user(&self, name: &str) {
        let p = Prefs {
            last_user: Some(name.to_string()),
        };
        if let Ok(s) = serde_json::to_string(&p) {
            let _ = write_atomic(&self.prefs_file(), &s, false);
        }
    }

    // ───── Favoritos (por usuário) ─────

    fn favs_file(&self) -> PathBuf {
        self.dir.join("favorites.json")
    }

    fn all_favs(&self) -> HashMap<String, Vec<String>> {
        fs::read_to_string(self.favs_file())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn favorites(&self, name: &str) -> Vec<String> {
        self.all_favs()
            .remove(&name.to_ascii_lowercase())
            .unwrap_or_default()
    }

    pub fn save_favorites(&self, name: &str, favs: &[String]) {
        let mut all = self.all_favs();
        all.insert(name.to_ascii_lowercase(), favs.to_vec());
        if let Ok(s) = serde_json::to_string_pretty(&all) {
            let _ = write_atomic(&self.favs_file(), &s, false);
        }
    }

    fn avatar_path(&self, name: &str) -> PathBuf {
        // o nome já foi validado no cadastro (só letras, números, _ - .)
        self.dir
            .join("avatars")
            .join(format!("{}.png", name.to_ascii_lowercase()))
    }

    /// Recorta no centro, redimensiona para 256x256 e salva como PNG.
    pub fn set_avatar(&self, name: &str, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() > 20 * 1024 * 1024 {
            return Err("Imagem muito grande (máx. 20 MB)".into());
        }
        let img = image::load_from_memory(bytes).map_err(|_| "Imagem inválida".to_string())?;
        let (w, h) = (img.width(), img.height());
        let s = w.min(h).max(1);
        let sq = img
            .crop_imm((w - s) / 2, (h - s) / 2, s, s)
            .resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
        sq.save_with_format(self.avatar_path(name), image::ImageFormat::Png)
            .map_err(|_| "Não foi possível salvar a foto".to_string())
    }

    pub fn avatar_bytes(&self, name: &str) -> Option<Vec<u8>> {
        fs::read(self.avatar_path(name)).ok()
    }
}

fn getrandom_fill(buf: &mut [u8]) -> Result<(), ()> {
    use rand_core::RngCore;
    OsRng.try_fill_bytes(buf).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store(tag: &str) -> (Store, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "cyberdesk-test-{}-{}-{}",
            tag,
            std::process::id(),
            now_secs()
        ));
        let _ = fs::remove_dir_all(&dir);
        (Store::open(dir.clone()), dir)
    }

    #[test]
    fn login_persiste_entre_aberturas_do_app() {
        let (mut s, dir) = temp_store("persist");
        s.register("maria", "segredo123").unwrap();
        assert!(s.login("maria", "segredo123").is_ok());
        assert!(s.login("maria", "errada").is_err());

        // "manter conectado" marcado
        s.remember("maria", true);
        assert!(s.session_days_left().unwrap() >= SESSION_DAYS - 1);

        // fecha o app e abre de novo: uma instância nova lê do disco
        let mut reaberto = Store::open(dir.clone());
        assert_eq!(reaberto.restore_session().as_deref(), Some("maria"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn sem_manter_conectado_nao_restaura() {
        let (mut s, dir) = temp_store("nosave");
        s.register("joao", "senha-forte").unwrap();
        s.remember("joao", false);
        let mut reaberto = Store::open(dir.clone());
        assert!(reaberto.restore_session().is_none());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn sair_apaga_a_sessao() {
        let (mut s, dir) = temp_store("logout");
        s.register("ana", "minhasenha").unwrap();
        s.remember("ana", true);
        s.forget("ana");
        assert!(s.session_days_left().is_none());
        let mut reaberto = Store::open(dir.clone());
        assert!(reaberto.restore_session().is_none());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn token_adulterado_ou_vencido_e_recusado() {
        let (mut s, dir) = temp_store("tamper");
        s.register("leo", "senha-do-leo").unwrap();
        s.remember("leo", true);
        let path = dir.join("session.json");

        // token trocado por outro valor
        let mut f: SessionFile = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let original = f.token.clone();
        f.token = "00".repeat(32);
        fs::write(&path, serde_json::to_string(&f).unwrap()).unwrap();
        assert!(Store::open(dir.clone()).restore_session().is_none());

        // token certo, mas prazo vencido
        f.token = original;
        f.expires = now_secs() - 10;
        fs::write(&path, serde_json::to_string(&f).unwrap()).unwrap();
        assert!(Store::open(dir.clone()).restore_session().is_none());
        assert!(!path.exists(), "sessão vencida deve ser apagada");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn sessao_antiga_sem_prazo_ainda_funciona_e_ganha_prazo() {
        let (mut s, dir) = temp_store("legacy");
        s.register("bia", "senha-da-bia").unwrap();
        s.remember("bia", true);
        let path = dir.join("session.json");
        let mut f: SessionFile = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        f.expires = 0; // formato da v0.3
        fs::write(&path, serde_json::to_string(&f).unwrap()).unwrap();

        let mut reaberto = Store::open(dir.clone());
        assert_eq!(reaberto.restore_session().as_deref(), Some("bia"));
        let novo: SessionFile = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(novo.expires > now_secs());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn lembra_o_ultimo_usuario() {
        let (mut s, dir) = temp_store("last");
        s.register("duda", "senha-da-duda").unwrap();
        assert!(s.last_user().is_none());
        s.set_last_user("duda");
        assert_eq!(Store::open(dir.clone()).last_user().as_deref(), Some("duda"));
        let _ = fs::remove_dir_all(dir);
    }
}
