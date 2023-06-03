use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Print the default config and exit.
    #[arg(long)]
    pub print_default_config: Option<Option<String>>,
}
