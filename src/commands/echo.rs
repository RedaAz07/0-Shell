pub fn echo(args: Vec<String>) {
    let buffer = args.join(" ");
    let cleaned = if buffer.starts_with('"') && buffer.ends_with('"') {
        buffer[1..buffer.len() - 1].to_string()
    } else {
        buffer
    };

    println!("{}", cleaned);
}
