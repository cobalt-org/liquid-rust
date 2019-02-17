#[test]
fn test_slugify_default() {
    assert_eq!(
        v!("q-bert-says"),
        jekyll_filters!(Slugify, v!(" Q*bert says @!#@!"))
    );
}

#[test]
fn test_slugify_pretty() {
    assert_eq!(
        v!("q-bert-says-_@!-@!"),
        jekyll_filters!(Slugify, v!(" Q*bert says _@!#?@!"), v!("pretty"))
    );
}
