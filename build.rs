fn main() {
    #[cfg(target_os = "windows")]
    {
        let _ = embed_resource::compile("windns.rc", embed_resource::NONE);
    }
}
