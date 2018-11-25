use test_helper::*;

#[test]
fn test_inc() {
    assert_template_result!("0", "{%increment port %}", v!({}));
    assert_template_result!("0 1", "{%increment port %} {%increment port%}", v!({}));
    assert_template_result!("0 0 1 2 1",
      "{%increment port %} {%increment starboard%} {%increment port %} {%increment port%} {%increment starboard %}",
      v!({}));
}

#[test]
#[ignore]
fn test_dec() {
    assert_template_result!("9", "{%decrement port %}", v!({ "port": 10 }));
    assert_template_result!("-1 -2", "{%decrement port %} {%decrement port%}", v!({}));
    assert_template_result!("1 5 2 2 5",
      "{%increment port %} {%increment starboard%} {%increment port %} {%decrement port%} {%decrement starboard %}",
      v!({ "port": 1, "starboard": 5 }));
}
