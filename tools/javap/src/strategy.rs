use crate::cmd::{self, Cmd};
use crate::misc;
use crate::util;
use clap::ArgMatches;

pub fn choose(matches: &ArgMatches) -> Box<dyn Cmd> {
    if matches.is_present("line_number") {
        Box::new(cmd::factory::line_number())
    } else if matches.is_present("disassemble") {
        Box::new(cmd::factory::disassemble())
    } else {
        unimplemented!()
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
