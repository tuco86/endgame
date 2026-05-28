fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rerun-if-changed=assets/endgame.rc");
        println!("cargo:rerun-if-changed=assets/endgame.ico");
        embed_resource::compile("assets/endgame.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
