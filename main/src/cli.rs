use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {

    /// Path to config file
    #[clap(short, long, value_parser, default_value_t = String::from("config.yml"))]
    pub config: String

}