// Embute o ícone e as informações de versão no CyberDesk.exe (só no Windows).
fn main() {
    println!("cargo:rerun-if-changed=assets/icon/cyberdesk.ico");
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon/cyberdesk.ico");
        res.set("ProductName", "CyberDesk");
        res.set("FileDescription", "CyberDesk - catalogo de hardware para ciberseguranca");
        res.set("LegalCopyright", "CyberDesk");
        if let Err(e) = res.compile() {
            // não impede o build se o compilador de recursos não estiver disponível
            println!("cargo:warning=nao foi possivel embutir o icone: {e}");
        }
    }
}
