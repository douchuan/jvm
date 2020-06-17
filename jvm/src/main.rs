extern crate clap;
extern crate env_logger;

use clap::{App, Arg};

use vm;
use vm::runtime::{self, thread::MainThread};
use vm::util;

fn main() {
    env_logger::init();
    vm::init_vm();

    let matches = App::new("")
        .arg(
            Arg::with_name("cp")
                .long("cp")
                .help("class search path of directories and zip/jar files")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("classpath")
                .long("classpath")
                .help("class search path of directories and zip/jar files")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("MAIN_CLASS")
                .help("to execute a class")
                .required(true)
                .index(1),
        )
        .arg(Arg::with_name("ARGS").multiple(true).help("[args...]"))
        .get_matches();

    let cp = matches.value_of("cp");
    if let Some(cp) = cp {
        runtime::add_class_paths(cp);
    }

    let classpath = matches.value_of("classpath");
    if let Some(classpath) = classpath {
        runtime::add_class_path(classpath);
    }

    let class = matches.value_of_lossy("MAIN_CLASS").unwrap().to_string();
    let args = matches.values_of_lossy("ARGS");
    // println!("main class: {}, args: {:?}", class, args);
    let mut thread = MainThread::new(class.replace(".", util::FILE_SEP), args);
    thread.run();
}
