mod attributes;
mod class;
mod constant_pool;
mod fields;
mod methods;
mod reader;
mod signature;

pub use class::parse_class_file;
pub use reader::Error;
pub use signature::{ClassSignature, FieldSignature, MethodSignature};

/// Parse a Java class file from raw bytes.
///
/// # Example
/// ```ignore
/// use class_parser::parse;
/// let data = std::fs::read("HelloWorld.class").unwrap();
/// let cf = parse(&data).unwrap();
/// println!("version: {:?}", cf.version);
/// ```
pub fn parse(data: &[u8]) -> std::result::Result<classfile::ClassFile, Error> {
    let mut r = reader::Reader::new(data.to_vec());
    class::parse_class_file(&mut r)
}

/// Alias for `parse` — kept for backward compatibility with vm crate.
pub fn parse_class(data: &[u8]) -> std::result::Result<classfile::ClassFile, Error> {
    parse(data)
}
