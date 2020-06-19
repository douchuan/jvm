#![allow(unused)]

use crate::runtime::thread::JavaThread;
use std::fmt::Write;

pub fn print_stack_trace(jt: &JavaThread) {
    let mut w = String::new();

    let _ = writeln!(&mut w);
    for (count, it) in jt.frames.iter().enumerate().rev() {
        let frame = it.read().unwrap();
        let cls = frame.mir.method.class.get_class();
        let method_id = frame.mir.method.name.clone();
        let line_num = {
            let area = frame.area.read().unwrap();
            frame.mir.method.get_line_num(area.pc as u16)
        };

        let _ = writeln!(
            &mut w,
            "{}{}:{}(:{})",
            " ".repeat(jt.frames.len() - count),
            String::from_utf8_lossy(cls.name.as_slice()),
            String::from_utf8_lossy(method_id.as_slice()),
            line_num
        );
    }

    error!("{}", w);
}
