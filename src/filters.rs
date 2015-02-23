pub fn size<'a>(input : &'a str) -> String{
    input.len().to_string()
}

#[test]
fn test_size(){
    assert_eq!(size("abc"), "3");
    assert_eq!(size("this has 22 characters"), "22");
}
