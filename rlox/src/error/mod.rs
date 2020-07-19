
pub fn report(line: i32, context: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, context, message);
}

pub fn error(line: i32, message: &str) {
    report(line, "", message);
}
