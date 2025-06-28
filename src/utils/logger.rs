use log::LevelFilter;
use env_logger::Builder;

pub fn init_logger(verbose: bool) {
    let level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    Builder::from_default_env()
        .filter_level(level)
        .init();
}