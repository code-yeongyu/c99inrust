pub(super) fn byte_sized(name: &str) -> Option<&'static str> {
    match name {
        "byte" | "lighttable_t" => Some("byte"),
        _ => None,
    }
}
