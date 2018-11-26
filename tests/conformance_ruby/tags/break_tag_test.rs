use test_helper::*;

// tests that no weird errors are raised if break is called outside of a
// block
#[test]
fn test_break_with_no_block() {
    let assigns = v!({ "i": 1 });
    let markup = "{% break %}";
    let expected = "";

    assert_template_result!(expected, markup, assigns);
}
