use class_parser::parse;
use classfile::ConstantPoolType;
use std::fs;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("FIXTURES_DIR") {
        return PathBuf::from(dir);
    }
    // Fallback for manual runs outside of build.rs
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn parse_fixture(name: &str) -> classfile::ClassFile {
    let path = fixture_dir().join(format!("{}.class", name));
    let data =
        fs::read(&path).unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    parse(&data).unwrap_or_else(|e| panic!("failed to parse {}: {:?}", path.display(), e))
}

#[test]
fn hello_world_magic_version() {
    let cf = parse_fixture("HelloWorld");
    assert!(cf.version.major >= 61, "should be Java 17+");
}

#[test]
fn hello_world_has_main_method() {
    let cf = parse_fixture("HelloWorld");
    let main = cf.methods.iter().find(|m| {
        let name = classfile::constant_pool::get_utf8(&cf.cp, m.name_index as usize);
        name.as_slice() == b"main"
    });
    assert!(main.is_some(), "HelloWorld should have a main method");
}

#[test]
fn simple_calc_method_count() {
    let cf = parse_fixture("SimpleCalc");
    // <init>, add, multiply, divide, subtract = 5 methods
    assert_eq!(cf.methods.len(), 5);
}

#[test]
fn simple_calc_field_count() {
    let cf = parse_fixture("SimpleCalc");
    assert_eq!(cf.fields.len(), 0); // no fields
}

#[test]
fn all_types_fields() {
    let cf = parse_fixture("AllTypes");
    // b, s, i, l, f, d, c, z, str, obj, arr, strs = 12 fields
    assert_eq!(cf.fields.len(), 12);

    // Check first field is byte
    let desc = classfile::constant_pool::get_utf8(&cf.cp, cf.fields[0].desc_index as usize);
    assert_eq!(desc.as_slice(), b"B");
}

#[test]
fn all_types_constant_pool_entries() {
    let cf = parse_fixture("AllTypes");
    let utf8_count = cf
        .cp
        .iter()
        .filter(|e| matches!(*e, ConstantPoolType::Utf8 { .. }))
        .count();
    let class_count = cf
        .cp
        .iter()
        .filter(|e| matches!(*e, ConstantPoolType::Class { .. }))
        .count();
    assert!(utf8_count > 10, "should have many Utf8 entries");
    assert!(class_count > 0, "should have at least one Class entry");
}

#[test]
fn interfaces_implements_interface() {
    let cf = parse_fixture("Interfaces");
    assert_eq!(cf.interfaces.len(), 1);
    assert!(cf.methods.len() >= 2); // <init> + run
}

#[test]
fn bad_magic_error() {
    let result = parse(&[0xDE, 0xAD, 0xBE, 0xEF, 0, 0, 0, 65]);
    assert!(result.is_err());
}

#[test]
fn truncated_file_error() {
    let result = parse(&[0xCA, 0xFE, 0xBA, 0xBE]);
    assert!(result.is_err());
}

#[test]
fn constant_pool_index_zero_is_unused() {
    let cf = parse_fixture("HelloWorld");
    assert!(cf.cp.get(0).is_some()); // index 0 is Nop
}

#[test]
fn simple_calc_has_constant_pool() {
    let cf = parse_fixture("SimpleCalc");
    assert!(
        cf.cp.len() > 5,
        "SimpleCalc should have a non-trivial constant pool"
    );
    let utf8_count = cf
        .cp
        .iter()
        .filter(|e| matches!(*e, ConstantPoolType::Utf8 { .. }))
        .count();
    assert!(utf8_count > 2);
}
