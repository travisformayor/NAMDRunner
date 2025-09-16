use namdrunner_lib::validation::shell;

fn main() {
    let result = shell::escape_parameter("file'quote'");
    println!("Result: {}", result);
    println!("Starts with ': {}", result.starts_with('\''));
    println!("Ends with ': {}", result.ends_with('\''));
    let inner = &result[1..result.len()-1];
    println!("Inner: {}", inner);
    println!("Contains '\": {}", inner.contains("'\""));
}
