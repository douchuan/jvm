use crate::cmd::{Cmd, Disassemble};
use crate::misc;
use crate::util;
use crate::Opt;

pub fn setup_from_opt(opt: &Opt) {
    let mut added = std::collections::HashSet::new();

    for v in [&opt.cp, &opt.classpath] {
        let paths = v.split(util::PATH_SEP);
        paths.for_each(|path| {
            if !added.contains(path) {
                misc::add_cp_path(path);
                added.insert(path);
            }
        });
    }
}

pub fn choose_from_opt(opt: &Opt) -> Box<dyn Cmd> {
    let d = Disassemble::new(opt);
    Box::new(d)
}
