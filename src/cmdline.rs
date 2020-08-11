
/// Command line configuration information
#[derive(Debug)]
pub struct ConfigInfo {
    pub log_file_name : String,
    pub site : String,
    pub console_log : bool,
    pub cert : String,
    pub key : String,
    pub port : u16
}

/// Loads information from the command line
pub fn get_command_line() -> ConfigInfo {
    let matches = clap_app!(rtinywiki =>
        (version: "0.0.0")
        (author: "Johnt <johnt7@gmail.com>")
        (about: "Rust Tiny Wiki")
        (@arg logfile: -l --log +takes_value "Log file name")
        (@arg cert: -c --cert +takes_value "Server certificate file")
        (@arg site: -s --site +takes_value "Location of site directory")
        (@arg key: -k --key +takes_value "Server key file")
        (@arg port: -p --port +takes_value "Port number")
        (@arg consoleLog: -d --dump "Dump log to console?")
    ).get_matches();
    ConfigInfo {
        log_file_name : matches.value_of("logfile").unwrap_or("site/wikiserver.log").to_string(),
        site : matches.value_of("site").unwrap_or("site").to_string(),
        cert : matches.value_of("cert").unwrap_or("server.cert").to_string(),
        key : matches.value_of("key").unwrap_or("server.key").to_string(),
        console_log : !matches.is_present("consoleLog"),
        port : matches.value_of("port").unwrap_or("9990").parse().unwrap_or(9990)
    }
}
