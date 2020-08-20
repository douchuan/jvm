use clap::Clap;

#[derive(Clap, Debug)]
#[clap(version)]
pub struct Opt {
    /// class search path of directories and zip/jar files
    #[clap(long)]
    pub cp: Option<String>,

    /// class search path of directories and zip/jar files
    #[clap(long)]
    pub classpath: Option<String>,

    #[clap(required = true)]
    pub class: String,

    pub args: Vec<String>,
}

pub fn parse() -> Opt {
    Opt::parse()
}
