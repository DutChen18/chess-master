pub struct Options {
    pub debug: bool,
}

impl Options {
    pub fn new() -> Self {
        Self {
            debug: false,
        }
    }
}
