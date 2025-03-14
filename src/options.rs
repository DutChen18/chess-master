pub struct Options {
    pub debug: bool,
    pub ownbook: bool,
}

impl Options {
    pub fn new() -> Self {
        Self {
            debug: false,
            ownbook: true,
        }
    }
}

