use clap::{Arg, ArgMatches, Command};
pub async fn get_envs() -> anyhow::Result<ArgMatches> {
    let matches = Command::new("data manager")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("Simple command line producer")
        .arg(
            Arg::new("log-conf")
                .long("log-conf")
                .help("Configure the logging format (example: 'rdkafka=trace')")
                .takes_value(true),
        )
        .get_matches();
    Ok(matches)
}
