#[cfg(test)]
mod tests {
    use crate::{
        config::{Builder, Config},
    };

    #[test]
    fn config_test() {
        let config = Builder::new()
            .enable_hook_manager()
            .set_database_name("asd".to_string())
            .build();

        let config2 = Config {
            db_name: "asd".to_string(),
            start_hook_manager: true,
        };
        
        assert_eq!(config2, config);
    }
}
