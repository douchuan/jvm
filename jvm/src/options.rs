use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Opt {
    /// class search path of directories and zip/jar files
    #[arg(long)]
    pub cp: Option<String>,

    /// class search path of directories and zip/jar files
    #[arg(long)]
    pub classpath: Option<String>,

    #[arg(required = true)]
    pub class: String,

    pub args: Vec<String>,
}

pub fn parse() -> Opt {
    Opt::parse()
}
