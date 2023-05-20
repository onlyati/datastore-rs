#[derive(Clone, PartialEq, Debug)]
pub struct Config {
    pub db_name: String,
    pub start_hook_manager: bool,
}

pub struct Builder {
    config: Config,
}

impl Builder {
    pub fn new() -> Self {
        return Builder {
            config: Config {
                db_name: "root".to_string(),
                start_hook_manager: false,
            },
        };
    }

    pub fn enable_hook_manager(&mut self) -> &mut Self {
        self.config.start_hook_manager = true;
        return self;
    }

    pub fn disable_hook_manager(&mut self) -> &mut Self {
        self.config.start_hook_manager = false;
        return self;
    }

    pub fn set_database_name(&mut self, name: String) -> &mut Self {
        self.config.db_name = name;
        return self;
    }

    pub fn build(&self) -> Config {
        return self.config.clone();
    }
}
