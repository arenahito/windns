fn main() {
    #[cfg(target_os = "windows")]
    {
        let _ = embed_resource::compile("windns.manifest", embed_resource::NONE);
        let _ = embed_resource::compile("windns.rc", embed_resource::NONE);
    }
}
