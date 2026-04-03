pub mod client;
pub mod model;
pub mod parse;

const DEFAULT_HOST: &str = "localhost";
const DEFAULT_PORT: &str = "11300";

/// Normalize a host:port input string.
///
/// - `None` or empty -> `"localhost:11300"`
/// - `":1234"` -> `"localhost:1234"`
/// - `"myhost"` -> `"myhost:11300"`
/// - `"myhost:1234"` -> `"myhost:1234"`
pub fn resolve_addr(input: Option<&str>) -> String {
    let input = input.unwrap_or("").trim();
    if input.is_empty() {
        return format!("{DEFAULT_HOST}:{DEFAULT_PORT}");
    }
    if input.starts_with(':') {
        return format!("{DEFAULT_HOST}{input}");
    }
    if input.contains(':') {
        return input.to_string();
    }
    format!("{input}:{DEFAULT_PORT}")
}
