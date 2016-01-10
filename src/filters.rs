pub fn size<'a>(input : &'a str) -> String{
    input.len().to_string()
}

pub fn upcase<'a>(input: &'a str) -> String {
    let str = String::from(input);
    str.to_uppercase()
}

#[test]
fn test_size(){
    assert_eq!(size("abc"), "3");
    assert_eq!(size("this has 22 characters"), "22");
}

#[test]
fn test_upcase() {
    assert_eq!(upcase("abc"), "ABC");
    assert_eq!(upcase("Hello World 21"), "HELLO WORLD 21");
}
