use std::borrow::Cow;
fn remove_whitespace(input: &str) -> Cow<str> {
    if input.contains(' ') {
        Cow::Owned(input.replace(' ', ""))
    } else {
        Cow::Borrowed(input)
    }
}

fn main() {
    let name = "Neeraj Chand";
    println!("{}", remove_whitespace(name));
}