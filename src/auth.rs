//! Login local: usuários e senhas ficam só neste computador.
//! Senhas são guardadas como hash Argon2id (nunca em texto puro).

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Serialize, Deserialize, Clone)]
struct User {
    name: String,
    hash: String,
}

#[derive(Serialize, Deserialize, Default)]
struct Db {
    users: Vec<User>,
}

pub struct Store {
    dir: PathBuf,
    db: Db,
}

fn data_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_DATA_HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("CyberDesk")
}

impl Store {
    pub fn load() -> Self {
        let dir = data_dir();
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
        Self { dir, db }
    }

    pub fn has_users(&self) -> bool {
        !self.db.users.is_empty()
    }

    fn save(&self) -> Result<(), String> {
        let s = serde_json::to_string_pretty(&self.db).map_err(|e| e.to_string())?;
        let file = self.dir.join("users.json");
        let tmp = self.dir.join("users.json.tmp");
        fs::write(&tmp, s).map_err(|_| "Não foi possível salvar os dados".to_string())?;
        fs::rename(&tmp, &file).map_err(|_| "Não foi possível salvar os dados".to_string())
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
