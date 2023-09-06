use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(long, short = 'c', alias = "config-file")]
    pub config_file_path: String,
}

impl Cli {
    pub fn init() -> Self {
        Self::parse()
    }
}
