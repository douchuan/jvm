use crate::cmd::{Cmd, Disassemble};
use crate::misc;
use crate::util;
use clap::ArgMatches;

pub fn choose(m: &ArgMatches) -> Box<dyn Cmd> {
    match Disassemble::new(m) {
        Some(d) => Box::new(d),
        None => unimplemented!(),
    }
}

pub fn setup_classpath(matches: &ArgMatches) {
    let mut added = std::collections::HashSet::new();

    vec![
        matches.value_of("cp").unwrap(),
        matches.value_of("classpath").unwrap(),
    ]
    .iter()
    .for_each(|&v| {
        let paths = v.split(util::PATH_SEP);
        paths.for_each(|path| {
            if !added.contains(path) {
                misc::add_cp_path(path);
                added.insert(path);
            }
        });
    });
}
