use liquid;
#[test]
fn test_slugify_default() {
    assert_eq!(
        v!("q-bert-says"),
        filters!(slugify, v!(" Q*bert says @!#@!"))
    );
}

#[test]
fn test_slugify_pretty() {
    assert_eq!(
        v!("q-bert-says-_@!-@!"),
        filters!(slugify, v!(" Q*bert says _@!#?@!"), v!("pretty"))
    );
}
