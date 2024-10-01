use log::LevelFilter;

pub fn env_logger_init() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        // .filter_module("overlord", LevelFilter::Warn)
        // .filter_module("vintage_consensus", LevelFilter::Warn)
        // .filter_module("vintage_network", LevelFilter::Warn)
        // .filter_module("vintage_proxy", LevelFilter::Warn)
        // .filter_module("vintage_blockchain", LevelFilter::Warn)
        .init();
}
