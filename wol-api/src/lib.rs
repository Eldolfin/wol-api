pub mod agent;
pub mod cache;
pub mod config;
pub mod consts;
pub mod machine;
pub mod utils;

pub mod misc {
    use directories::ProjectDirs;
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref dirs: ProjectDirs = ProjectDirs::from("top", "eldolfin", "wol-api")
            .expect("to be able to have project dirs");
    }
}

pub mod test {
    use std::sync::Once;

    use rstest::fixture;

    #[fixture]
    pub fn logfxt() {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            env_logger::builder()
                .format_timestamp(None)
                .is_test(true)
                .init();
        });
    }
}
