mod options;

use tracing_subscriber;
use vm;
use vm::runtime::{self, thread::MainThread};
use vm::util;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
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
    let mut thread = MainThread::new(class.replace(".", util::FILE_SEP), args);
    thread.run();
}
