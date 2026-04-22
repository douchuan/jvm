mod cmd;
mod misc;
mod sd;
mod strategy;
mod template;
mod trans;
mod util;

use clap::Parser;
use class_parser::parse_class;

/*
Usage: javap <options> <classes>
where possible options include:
  -help  --help  -?        Print this usage message
  -version                 Version information
  -v  -verbose             Print additional information
  -l                       Print line number and local variable tables
  -public                  Show only public classes and members
  -protected               Show protected/public classes and members
  -package                 Show package/protected/public classes
                           and members (default)
  -p  -private             Show all classes and members
  -c                       Disassemble the code
  -s                       Print internal type signatures
  -sysinfo                 Show system info (path, size, date, MD5 hash)
                           of class being processed
  -constants               Show final constants
  -classpath <path>        Specify where to find user class files
  -cp <path>               Specify where to find user class files
  -bootclasspath <path>    Override location of bootstrap class files
*/

#[derive(Parser, Debug)]
#[command(about = "")]
struct Opt {
    #[arg(long)]
    version: bool,

    #[arg(long, short = 'v')]
    verbose: bool,

    #[arg(short = 'l')]
    line_number: bool,

    #[arg(long, conflicts_with_all = ["protected", "package", "private"])]
    public: bool,

    #[arg(long)]
    protected: bool,

    #[arg(long)]
    package: bool,

    #[arg(long, short = 'p')]
    private: bool,

    #[arg(short = 'c')]
    disassemble: bool,

    #[arg(short = 's')]
    signatures: bool,

    #[arg(long)]
    sysinfo: bool,

    #[arg(long)]
    constants: bool,

    #[arg(long, default_value = ".")]
    cp: String,

    #[arg(long, default_value = ".")]
    classpath: String,

    #[arg(required = true)]
    classes: Vec<String>,
}

fn main() {
    init();

    let opt = Opt::parse();

    strategy::setup_from_opt(&opt);

    if opt.version {
        println!(env!("CARGO_PKG_VERSION"));
    }

    let commander = strategy::choose_from_opt(&opt);

    for it in &opt.classes {
        match misc::find_class(it) {
            Ok(r) => {
                if let Ok(cf) = parse_class(&r.1) {
                    commander.run(&r.0, cf);
                } else {
                    tracing::error!("parse class error: {}", it);
                }
            }
            Err(_e) => {
                println!("Error: class not found: {}", it);
            }
        }
    }
}

fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    misc::cp_manager_init();
}
