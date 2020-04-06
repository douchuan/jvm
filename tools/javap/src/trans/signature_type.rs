use classfile::SignatureType;

pub trait Translator {
    fn into_string(&self) -> String;
}

impl Translator for SignatureType {
    fn into_string(&self) -> String {
        match self {
            SignatureType::Int => "int".into(),
            SignatureType::Byte => "byte".into(),
            SignatureType::Char => "char".into(),
            SignatureType::Double => "double".into(),
            SignatureType::Float => "float".into(),
            SignatureType::Long => "long".into(),
            SignatureType::Object(desc, _) => to_java_style(&desc),
            SignatureType::Short => "short".into(),
            SignatureType::Boolean => "boolean".into(),
            SignatureType::Array(desc) => to_java_style(&desc),
            SignatureType::Void => "void".into(),
        }
    }
}

fn to_java_style(desc: &[u8]) -> String {
    //calc array dimensions
    let mut i = 0;
    while desc[i] == b'[' {
        i += 1;
    }
    let ary_size = i;

    let mut name = if desc[i] == b'L' {
        let desc = &desc[(i + 1)..(desc.len() - 1)];
        String::from_utf8_lossy(desc).replace("/", ".")
    } else {
        match desc[i] {
            b'B' => "byte".into(),
            b'C' => "char".into(),
            b'D' => "double".into(),
            b'F' => "float".into(),
            b'I' => "int".into(),
            b'J' => "long".into(),
            b'S' => "short".into(),
            b'Z' => "boolean".into(),
            _ => unreachable!(),
        }
    };

    for _ in 0..ary_size {
        name.push_str("[]");
    }

    name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_parse() {
        let tests = vec![
            ("Ljava/lang/Object;", "java.lang.Object"),
            ("[Ljava/lang/Object;", "java.lang.Object[]"),
            ("[[Ljava/lang/Object;", "java.lang.Object[][]"),
            ("[B", "byte[]"),
            ("[C", "char[]"),
            ("[D", "double[]"),
            ("[F", "float[]"),
            ("[J", "long[]"),
            ("[S", "short[]"),
            ("[Z", "boolean[]"),
        ];

        for it in tests.iter() {
            assert_eq!(to_java_style(it.0.as_bytes()), it.1);
        }
    }
}
