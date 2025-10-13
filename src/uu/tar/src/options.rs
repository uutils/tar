struct TarOptions {
    block_size: usize,
}

impl Default for TarOptions {
    fn default() -> TarOptions {
        Self {
            block_size: 512
        }
    }
}
