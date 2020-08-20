extern crate clap;
extern crate env_logger;

mod options;

use vm;
use vm::runtime::{self, thread::MainThread};
use vm::util;

fn main() {
    env_logger::init();
    vm::init_vm();

    let opt = options::parse();

    if let Some(cp) = &opt.cp {
        runtime::add_class_paths(cp);
    }

    if let Some(classpath) = &opt.classpath {
        runtime::add_class_path(classpath);
    }

    let class = opt.class;
    let args = opt.args;
    // println!("main class: {}, args: {:?}", class, args);
    let mut thread = MainThread::new(class.replace(".", util::FILE_SEP), args);
    thread.run();
}
