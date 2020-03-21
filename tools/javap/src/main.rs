#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

extern crate clap;
extern crate env_logger;

mod cmd;
mod misc;
mod strategy;
mod trans;
mod util;

use clap::{App, Arg, ArgMatches};
use class_parser::parse_class;
use cmd::Cmd;

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

//todo: can clap support '-private' style option?
fn main() {
    init();

    let matches = App::new("")
        .arg(
            Arg::with_name("version")
                .long("version")
                .help("Print this usage message"),
        )
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .help("Print additional information"),
        )
        .arg(
            Arg::with_name("line_number")
                .short("l")
                .help("Print line number and local variable tables"),
        )
        .arg(
            Arg::with_name("public")
                .long("public")
                .help("Show only public classes and members"),
        )
        .arg(
            Arg::with_name("protected")
                .long("protected")
                .help("Show protected/public classes and members"),
        )
        .arg(
            Arg::with_name("package")
                .long("package")
                .help("Show package/protected/public classes\nand members (default)"),
        )
        .arg(
            Arg::with_name("private")
                .long("private")
                .short("p")
                .help("Show all classes and members"),
        )
        .arg(
            Arg::with_name("disassemble")
                .short("c")
                .help("Disassemble the code"),
        )
        .arg(
            Arg::with_name("signatures")
                .short("s")
                .help("Print internal type signatures"),
        )
        .arg(
            Arg::with_name("sysinfo")
                .long("sysinfo")
                .help("Show system info (path, size, date, MD5 hash)\nof class being processed"),
        )
        .arg(
            Arg::with_name("constants")
                .long("constants")
                .help("Show final constants"),
        )
        .arg(
            Arg::with_name("cp")
                .long("cp")
                .help("Specify where to find user class files")
                .default_value(".")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("classpath")
                .long("classpath")
                .help("Specify where to find user class files")
                .default_value(".")
                .takes_value(true),
        )
        .arg(Arg::with_name("classes").multiple(true).index(1))
        .get_matches();

    strategy::setup_classpath(&matches);

    let commander = strategy::choose(&matches);

    let classes = matches.values_of("classes").unwrap();
    for it in classes {
        let _ = misc::find_class(it).and_then(|r| {
            let _ = parse_class(&r.1).and_then(|(_, cf)| {
                commander.run(cf);
                Ok(())
            });

            Ok(())
        });
    }
}

fn init() {
    env_logger::init();
    misc::cp_manager_init();
}
