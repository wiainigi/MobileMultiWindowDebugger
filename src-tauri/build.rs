fn main() {
    // tauri-build 默认只监视 tauri.conf.json 与 capabilities，
    // 这里补上图标目录：替换应用图标后仍需重新写入 exe 资源段。
    println!("cargo:rerun-if-changed=icons");
    tauri_build::build()
}
