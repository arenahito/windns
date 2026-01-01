fn main() {
    #[cfg(target_os = "windows")]
    {
        let _ = embed_resource::compile("windns.manifest", embed_resource::NONE);
    }
}
