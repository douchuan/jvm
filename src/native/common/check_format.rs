
pub fn is_valid_class_name(s: &String) -> bool {
    if s.contains("/") {
        return false;
    }

    true
}