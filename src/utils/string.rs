pub fn escape_sh(value: &str) -> String {
    value
        .replace('$', "\\$")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('"', "\\\"")
        .replace('\'', "\\\'")
}
