
use simplelog::*;
use std::fs::File;
use std::io::Write;

pub fn init_logger() {
    let log_file = File::create("log.log").expect("Failed to create log file");

    CombinedLogger::init(vec![
        WriteLogger::new(LevelFilter::Info, Config::default(), log_file),
    ])
    .expect("Failed to initialize logger");
}
