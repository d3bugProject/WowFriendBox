fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let res_o = format!("{}/resource.o", out_dir);
        let mut res = winres::WindowsResource::new();
        res.set_icon("app.ico");
        res.set("ProductName", "WowFriendBox");
        res.set("FileDescription", "WowFriendBox - Multiboxing Controller for WoW 3.3.5a");
        res.set("LegalCopyright", "Copyright (C) 2026");
        res.compile().unwrap();

        if std::path::Path::new(&res_o).exists() {
            println!("cargo:rustc-link-arg={}", res_o);
        }
    }
}
