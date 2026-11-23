//! Atualização do app pelas Releases do GitHub.
//!
//! Cada push na `main` publica uma Release `v<maior>.<menor>.<build>` com o
//! `CyberDesk.exe`. O número do build é gravado no executável na compilação
//! (variável `CYBERDESK_BUILD`, preenchida pelo workflow). O app compara o
//! seu build com o da última Release: se o do GitHub for maior, há atualização.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

const REPO: &str = "vargas-byb/CyberDesk";
const ASSET: &str = "CyberDesk.exe";
/// Abaixo disso o arquivo baixado certamente não é o app.
const MIN_SIZE: u64 = 1_000_000;

#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub build: u32,
    pub tag: String,
    pub url: String,
    pub size: u64,
    pub sha256: Option<String>,
}

impl Release {
    /// "v0.4.12" -> "0.4.12"
    pub fn version(&self) -> &str {
        self.tag.trim_start_matches('v')
    }
}

/// Número do build gravado na compilação (0 = compilação local, sem build).
pub fn build() -> u32 {
    option_env!("CYBERDESK_BUILD")
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

/// Versão mostrada na tela: "0.4.<build>" (ou a do Cargo.toml em builds locais).
pub fn version_label() -> String {
    match build() {
        0 => env!("CARGO_PKG_VERSION").to_string(),
        b => format!(
            "{}.{}.{}",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            b
        ),
    }
}

pub fn is_newer(release_build: u32) -> bool {
    release_build > build()
}

/// "v0.4.12" -> 12
fn parse_build(tag: &str) -> Option<u32> {
    tag.trim_start_matches('v').rsplit('.').next()?.parse().ok()
}

fn parse_release(json: &str) -> Result<Release, String> {
    let v: Value = serde_json::from_str(json).map_err(|_| "resposta inválida do GitHub".to_string())?;
    let tag = v["tag_name"].as_str().ok_or("Release sem versão")?.to_string();
    let build = parse_build(&tag).ok_or("versão da Release não reconhecida")?;
    let asset = v["assets"]
        .as_array()
        .and_then(|a| a.iter().find(|x| x["name"].as_str() == Some(ASSET)))
        .ok_or("a Release não tem o CyberDesk.exe")?;
    let url = asset["browser_download_url"]
        .as_str()
        .ok_or("Release sem link de download")?
        .to_string();
    let prefix = format!("https://github.com/{REPO}/releases/download/").to_lowercase();
    if !url.to_lowercase().starts_with(&prefix) {
        return Err("link de download inesperado".into());
    }
    let size = asset["size"].as_u64().unwrap_or(0);
    let sha256 = asset["digest"]
        .as_str()
        .and_then(|d| d.strip_prefix("sha256:"))
        .map(|d| d.to_lowercase());
    Ok(Release {
        build,
        tag,
        url,
        size,
        sha256,
    })
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .user_agent("CyberDesk-updater")
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .build()
}

fn net_err(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(403, _) | ureq::Error::Status(429, _) => {
            "o GitHub limitou as consultas, tente mais tarde".to_string()
        }
        ureq::Error::Status(c, _) => format!("o GitHub respondeu com erro {c}"),
        _ => "sem conexão com o GitHub".to_string(),
    }
}

/// Consulta a última Release publicada.
pub fn check() -> Result<Release, String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let body = agent()
        .get(&url)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(net_err)?
        .into_string()
        .map_err(|_| "resposta inválida do GitHub".to_string())?;
    parse_release(&body)
}

fn sibling(exe: &std::path::Path, suffix: &str) -> PathBuf {
    let mut name = exe.file_name().map(|n| n.to_os_string()).unwrap_or_default();
    name.push(suffix);
    exe.with_file_name(name)
}

/// Apaga restos de uma atualização anterior (o .exe antigo renomeado).
pub fn cleanup() {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::fs::remove_file(sibling(&exe, ".old"));
        let _ = std::fs::remove_file(sibling(&exe, ".new"));
    }
}

/// Baixa a Release, confere o arquivo, troca o executável atual e abre o novo.
/// Em caso de sucesso o chamador deve encerrar o processo.
pub fn install(rel: &Release, progress: &AtomicU32) -> Result<(), String> {
    if build() == 0 {
        return Err("build de desenvolvimento: baixe o .exe na aba Releases".into());
    }
    let exe = std::env::current_exe().map_err(|_| "não achei o caminho do app".to_string())?;
    let new = sibling(&exe, ".new");
    let old = sibling(&exe, ".old");

    let result = download(rel, &new, progress).and_then(|_| swap(&exe, &new, &old));
    if result.is_err() {
        let _ = std::fs::remove_file(&new);
    }
    result?;

    std::process::Command::new(&exe)
        .spawn()
        .map_err(|_| "atualizou, mas não consegui reabrir: abra o app de novo".to_string())?;
    Ok(())
}

fn download(rel: &Release, dest: &std::path::Path, progress: &AtomicU32) -> Result<(), String> {
    progress.store(0, Ordering::Relaxed);
    let resp = agent().get(&rel.url).call().map_err(net_err)?;
    let total = resp
        .header("Content-Length")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(rel.size);
    let mut reader = resp.into_reader();
    let mut file =
        std::fs::File::create(dest).map_err(|_| "sem permissão para gravar na pasta do app".to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    let mut done: u64 = 0;
    let mut head = [0u8; 2];
    loop {
        let n = reader.read(&mut buf).map_err(|_| "o download foi interrompido".to_string())?;
        if n == 0 {
            break;
        }
        if done == 0 && n >= 2 {
            head.copy_from_slice(&buf[..2]);
        }
        hasher.update(&buf[..n]);
        file.write_all(&buf[..n])
            .map_err(|_| "não consegui gravar o arquivo baixado".to_string())?;
        done += n as u64;
        if total > 0 {
            progress.store(((done * 100) / total).min(100) as u32, Ordering::Relaxed);
        }
    }
    file.flush().map_err(|_| "não consegui gravar o arquivo baixado".to_string())?;
    drop(file);

    if done < MIN_SIZE || (rel.size > 0 && done != rel.size) {
        return Err("o arquivo baixado veio incompleto".into());
    }
    if cfg!(windows) && &head != b"MZ" {
        return Err("o arquivo baixado não é um executável".into());
    }
    if let Some(want) = &rel.sha256 {
        let got: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if &got != want {
            return Err("o arquivo baixado não passou na verificação".into());
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755));
    }
    progress.store(100, Ordering::Relaxed);
    Ok(())
}

/// O Windows não deixa apagar um .exe em uso, mas deixa renomear: o atual vira
/// ".old", o novo assume o nome e o ".old" é apagado na próxima abertura.
fn swap(exe: &std::path::Path, new: &std::path::Path, old: &std::path::Path) -> Result<(), String> {
    let _ = std::fs::remove_file(old);
    std::fs::rename(exe, old).map_err(|_| "não consegui substituir o app (sem permissão?)".to_string())?;
    if std::fs::rename(new, exe).is_err() {
        let _ = std::fs::rename(old, exe); // desfaz
        return Err("não consegui colocar a nova versão no lugar".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "tag_name": "v0.4.12",
        "assets": [
            {"name": "outro.txt", "size": 5, "browser_download_url": "https://github.com/vargas-byb/CyberDesk/releases/download/v0.4.12/outro.txt"},
            {"name": "CyberDesk.exe", "size": 21105152,
             "digest": "sha256:ABCDEF0123",
             "browser_download_url": "https://github.com/vargas-byb/CyberDesk/releases/download/v0.4.12/CyberDesk.exe"}
        ]
    }"#;

    #[test]
    fn le_o_numero_do_build_da_tag() {
        assert_eq!(parse_build("v0.4.12"), Some(12));
        assert_eq!(parse_build("0.4.7"), Some(7));
        assert_eq!(parse_build("v0.4.x"), None);
    }

    #[test]
    fn le_a_release_do_github() {
        let r = parse_release(SAMPLE).unwrap();
        assert_eq!(r.build, 12);
        assert_eq!(r.version(), "0.4.12");
        assert_eq!(r.size, 21105152);
        assert_eq!(r.sha256.as_deref(), Some("abcdef0123"));
        assert!(r.url.ends_with("/v0.4.12/CyberDesk.exe"));
    }

    #[test]
    fn recusa_link_de_outro_lugar() {
        let ruim = SAMPLE.replace("github.com/vargas-byb", "exemplo.com/outro");
        assert!(parse_release(&ruim).is_err());
    }

    #[test]
    fn recusa_release_sem_exe() {
        let sem = SAMPLE.replace("\"CyberDesk.exe\"", "\"x.zip\"");
        assert!(parse_release(&sem).is_err());
    }

    #[test]
    fn troca_o_executavel_e_desfaz_se_falhar() {
        let dir = std::env::temp_dir().join(format!("cyberdesk-upd-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (exe, new, old) = (dir.join("app"), dir.join("app.new"), dir.join("app.old"));
        std::fs::write(&exe, b"antigo").unwrap();
        std::fs::write(&new, b"novo").unwrap();
        swap(&exe, &new, &old).unwrap();
        assert_eq!(std::fs::read(&exe).unwrap(), b"novo");
        assert_eq!(std::fs::read(&old).unwrap(), b"antigo");
        // sem o arquivo novo, o app antigo continua no lugar
        assert!(swap(&exe, &new, &old).is_err());
        assert_eq!(std::fs::read(&exe).unwrap(), b"novo");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
