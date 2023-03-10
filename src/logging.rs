use log::{debug, error, info, trace, warn, LevelFilter, SetLoggerError};

use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};



fn build_log_file(log_file: &str) -> FileAppender {
    // log file.
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(log_file)
        .expect("Failed to build logfile");

    return logfile
}

fn build_request_log_file(request_path: &str) -> FileAppender {
    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(request_path)
        .expect("Failed to build logfile");

    return requests
}


fn build_config_file(request: FileAppender,logfile: FileAppender) -> Config {

    let level = log::LevelFilter::Debug;
    // Log trace level output to file where trace is the default level
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(Appender::builder()
            .filter(Box::new(ThresholdFilter::new(level)))
            .build("requestsfile", Box::new(request)))
        .build(
            Root::builder()
                .appender("logfile")
                .appender("requestsfile")
                .build(LevelFilter::Trace)
        ).expect("Couldn't build logging config file");

    return config
}


pub fn set_logging(log_path: &str, requests_path: &str) -> log4rs::Handle{
    let logfile = build_log_file(log_path);
    let requestlog = build_request_log_file(requests_path);
    let config = build_config_file(logfile, requestlog);

    log4rs::init_config(config).expect("can make logging handle")
}


