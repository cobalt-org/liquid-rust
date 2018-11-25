use liquid;

use test_helper::*;

#[test]
fn test_size() {
    assert_eq!(v!(3), filters!(size, v!([1, 2, 3])));
    assert_eq!(v!(0), filters!(size, v!([])));
    assert_eq!(v!(0), filters!(size, v!(nil)));
}

#[test]
fn test_downcase() {
    assert_eq!(v!("testing"), filters!(downcase, v!("Testing")));
    assert_eq!(v!(""), filters!(downcase, Nil));
}

#[test]
fn test_upcase() {
    assert_eq!(v!("TESTING"), filters!(upcase, v!("Testing")));
    assert_eq!(v!(""), filters!(upcase, Nil));
}

#[test]
#[ignore]
fn test_slice() {
    assert_eq!(v!("oob"), filters!(slice, v!("foobar"), v!(1), v!(3)));
    assert_eq!(v!("oobar"), filters!(slice, v!("foobar"), v!(1), v!(1000)));
    assert_eq!(v!(""), filters!(slice, v!("foobar"), v!(1), v!(0)));
    assert_eq!(v!("o"), filters!(slice, v!("foobar"), v!(1), v!(1)));
    assert_eq!(v!("bar"), filters!(slice, v!("foobar"), v!(3), v!(3)));
    assert_eq!(v!("ar"), filters!(slice, v!("foobar"), v!(-2), v!(2)));
    assert_eq!(v!("ar"), filters!(slice, v!("foobar"), v!(-2), v!(1000)));
    assert_eq!(v!("r"), filters!(slice, v!("foobar"), v!(-1)));
    assert_eq!(v!(""), filters!(slice, Nil, v!(0)));
    assert_eq!(v!(""), filters!(slice, v!("foobar"), v!(100), v!(10)));
    assert_eq!(v!(""), filters!(slice, v!("foobar"), v!(-100), v!(10)));
    assert_eq!(v!("oob"), filters!(slice, v!("foobar"), v!("1"), v!("3")));
    filters_fail!(slice, v!("foobar"), Nil);
    filters_fail!(slice, v!("foobar"), v!(0), v!(""));
}

#[test]
#[ignore]
fn test_slice_on_arrays() {
    let input = v!(["f", "o", "o", "b", "a", "r"]);
    assert_eq!(v!(["o", "o", "b"]), filters!(slice, input, v!(1), v!(3)));
    assert_eq!(
        v!(["o", "o", "b", "a", "r"]),
        filters!(slice, input, v!(1), v!(1000))
    );
    assert_eq!(v!([]), filters!(slice, input, v!(1), v!(0)));
    assert_eq!(v!(["o"]), filters!(slice, input, v!(1), v!(1)));
    assert_eq!(v!(["b", "a", "r"]), filters!(slice, input, v!(3), v!(3)));
    assert_eq!(v!(["a", "r"]), filters!(slice, input, v!(-2), v!(2)));
    assert_eq!(v!(["a", "r"]), filters!(slice, input, v!(-2), v!(1000)));
    assert_eq!(v!(["r"]), filters!(slice, input, v!(-1)));
    assert_eq!(v!([]), filters!(slice, input, v!(100), v!(10)));
    assert_eq!(v!([]), filters!(slice, input, v!(-100), v!(10)));
}

#[test]
#[ignore]
fn test_truncate() {
    assert_eq!(v!("1234..."), filters!(truncate, v!("1234567890"), v!(7)));
    assert_eq!(
        v!("1234567890"),
        filters!(truncate, v!("1234567890"), v!(20))
    );
    assert_eq!(v!("..."), filters!(truncate, v!("1234567890"), v!(0)));
    assert_eq!(v!("1234567890"), filters!(truncate, v!("1234567890")));
    assert_eq!(
        v!("测试..."),
        filters!(truncate, v!("测试测试测试测试"), v!(5))
    );
    assert_eq!(
        v!("12341"),
        filters!(truncate, v!("1234567890"), v!(5), v!(1))
    );
}

#[test]
#[ignore]
fn test_split() {
    assert_eq!(v!(["12", "34"]), filters!(split, v!("12~34"), v!("~")));
    assert_eq!(
        v!(["A? ", " ,Z"]),
        filters!(split, v!("A? ~ ~ ~ ,Z"), v!("~ ~ ~"))
    );
    assert_eq!(v!(["A?Z"]), filters!(split, v!("A?Z"), v!("~")));
    assert_eq!(v!([]), filters!(split, Nil, v!(" ")));
    assert_eq!(v!(["A", "Z"]), filters!(split, v!("A1Z"), v!(1)));
}

#[test]
#[ignore]
fn test_escape() {
    assert_eq!(v!("&lt;strong&gt;"), filters!(escape, v!("<strong>")));
    assert_eq!(v!("1"), filters!(escape, v!(1)));
    assert_eq!(v!("2001-02-03"), filters!(escape, date(2001, 2, 3)));
    assert_eq!(Nil, filters!(escape, Nil));
}

#[test]
#[ignore]
fn test_h() {
    /*
    assert_eq!(v!("&lt;strong&gt;"), filters!(h, v!("<strong>")));
    assert_eq!(v!("1"), filters!(h, 1));
    assert_eq!(v!("2001-02-03"), filters!(h, date(2001, 2, 3)));
    assert_eq!(Nil, filters!(h, Nil));
    */
}

#[test]
fn test_escape_once() {
    assert_eq!(
        v!("&lt;strong&gt;Hulk&lt;/strong&gt;"),
        filters!(escape_once, v!("&lt;strong&gt;Hulk</strong>"))
    );
}

#[test]
#[ignore]
fn test_url_encode() {
    assert_eq!(
        v!("foo%2B1%40example.com"),
        filters!(url_encode, v!("foo+1@example.com"))
    );
    assert_eq!(v!("1"), filters!(url_encode, v!(1)));
    assert_eq!(v!("2001-02-03"), filters!(url_encode, date(2001, 2, 3)));
    assert_eq!(Nil, filters!(url_encode, Nil));
}

#[test]
#[ignore]
fn test_url_decode() {
    assert_eq!(v!("foo bar"), filters!(url_decode, v!("foo+bar")));
    assert_eq!(v!("foo bar"), filters!(url_decode, v!("foo%20bar")));
    assert_eq!(
        v!("foo+1@example.com"),
        filters!(url_decode, v!("foo%2B1%40example.com"))
    );
    assert_eq!(v!("1"), filters!(url_decode, v!(1)));
    assert_eq!(v!("2001-02-03"), filters!(url_decode, date(2001, 2, 3)));
    assert_eq!(Nil, filters!(url_decode, Nil));
}

#[test]
fn test_truncatewords() {
    assert_eq!(
        v!("one two three"),
        filters!(truncatewords, v!("one two three"), v!(4))
    );
    assert_eq!(
        v!("one two..."),
        filters!(truncatewords, v!("one two three"), v!(2))
    );
    assert_eq!(
        v!("one two three"),
        filters!(truncatewords, v!("one two three"))
    );
    assert_eq!(v!("Two small (13&#8221; x 5.5&#8221; x 10&#8221; high) baskets fit inside one large basket (13&#8221;..."), filters!(truncatewords, v!("Two small (13&#8221; x 5.5&#8221; x 10&#8221; high) baskets fit inside one large basket (13&#8221; x 16&#8221; x 10.5&#8221; high) with cover."), v!(15)));
    assert_eq!(
        v!("测试测试测试测试"),
        filters!(truncatewords, v!("测试测试测试测试"), v!(5))
    );
    assert_eq!(
        v!("one two1"),
        filters!(truncatewords, v!("one two three"), v!(2), v!(1))
    );
}

#[test]
fn test_strip_html() {
    assert_eq!(v!("test"), filters!(strip_html, v!(r#"<div>test</div>"#)));
    assert_eq!(
        v!("test"),
        filters!(strip_html, v!(r#"<div id="test">test</div>"#))
    );
    assert_eq!(
        v!(""),
        filters!(
            strip_html,
            v!(r#"<script type="text/javascript">document.write"some stuff";</script>"#)
        )
    );
    assert_eq!(
        v!(""),
        filters!(strip_html, v!(r#"<style type="text/css">foo bar</style>"#))
    );
    assert_eq!(
        v!("test"),
        filters!(strip_html, v!(r#"<div\nclass="multiline">test</div>"#))
    );
    assert_eq!(
        v!("test"),
        filters!(strip_html, v!(r#"<!-- foo bar \n test -->test"#))
    );
    assert_eq!(v!(""), filters!(strip_html, Nil));
}

#[test]
fn test_join() {
    assert_eq!(v!("1 2 3 4"), filters!(join, v!([1, 2, 3, 4])));
    assert_eq!(
        v!("1 - 2 - 3 - 4"),
        filters!(join, v!([1, 2, 3, 4]), v!(" - "))
    );
    assert_eq!(v!("1121314"), filters!(join, v!([1, 2, 3, 4]), v!(1)));
}

#[test]
#[ignore]
fn test_sort() {
    assert_eq!(v!([1, 2, 3, 4]), filters!(sort, v!([4, 3, 2, 1])));
    assert_eq!(
        v!([{ "a": 1 }, { "a": 2 }, { "a": 3 }, { "a": 4 }]),
        filters!(
            sort,
            v!([{ "a": 4 }, { "a": 3 }, { "a": 1 }, { "a": 2 }]),
            v!("a")
        )
    );
}

#[test]
#[ignore]
fn test_sort_with_nils() {
    assert_eq!(v!([1, 2, 3, 4, nil]), filters!(sort, v!([nil, 4, 3, 2, 1])));
    assert_eq!(
        v!([{ "a": 1 }, { "a": 2 }, { "a": 3 }, { "a": 4 }, {}]),
        filters!(
            sort,
            v!([{ "a": 4 }, { "a": 3 }, {}, { "a": 1 }, { "a": 2 }]),
            v!("a")
        )
    );
}

#[test]
#[ignore]
fn test_sort_when_property_is_sometimes_missing_puts_nils_last() {
    let input = v!([
      { "price": 4, "handle": "alpha" },
      { "handle": "beta" },
      { "price": 1, "handle": "gamma" },
      { "handle": "delta" },
      { "price": 2, "handle": "epsilon" }
    ]);
    let expectation = v!([
      { "price": 1, "handle": "gamma" },
      { "price": 2, "handle": "epsilon" },
      { "price": 4, "handle": "alpha" },
      { "handle": "delta" },
      { "handle": "beta" }
    ]);
    assert_eq!(expectation, filters!(sort, input, v!("price")));
}

#[test]
#[ignore]
fn test_sort_natural() {
    assert_eq!(
        v!(["a", "B", "c", "D"]),
        filters!(sort_natural, v!(["c", "D", "a", "B"]))
    );
    assert_eq!(
        v!([{ "a": "a" }, { "a": "B" }, { "a": "c" }, { "a": "D" }]),
        filters!(
            sort_natural,
            v!([{ "a": "D" }, { "a": "c" }, { "a": "a" }, { "a": "B" }]),
            v!("a")
        )
    );
}

#[test]
#[ignore]
fn test_sort_natural_with_nils() {
    assert_eq!(
        v!(["a", "B", "c", "D", nil]),
        filters!(sort_natural, v!([nil, "c", "D", "a", "B"]))
    );
    assert_eq!(
        v!([{ "a": "a" }, { "a": "B" }, { "a": "c" }, { "a": "D" }, {}]),
        filters!(
            sort_natural,
            v!([{ "a": "D" }, { "a": "c" }, {}, { "a": "a" }, { "a": "B" }]),
            v!("a")
        )
    );
}

#[test]
#[ignore]
fn test_sort_natural_when_property_is_sometimes_missing_puts_nils_last() {
    let input = v!([
      { "price": "4", "handle": "alpha" },
      { "handle": "beta" },
      { "price": "1", "handle": "gamma" },
      { "handle": "delta" },
      { "price": 2, "handle": "epsilon" }
    ]);
    let expectation = v!([
      { "price": "1", "handle": "gamma" },
      { "price": 2, "handle": "epsilon" },
      { "price": "4", "handle": "alpha" },
      { "handle": "delta" },
      { "handle": "beta" }
    ]);
    assert_eq!(expectation, filters!(sort_natural, input, v!("price")));
}

#[test]
#[ignore]
fn test_sort_natural_case_check() {
    let input = v!([
      { "key": "X" },
      { "key": "Y" },
      { "key": "Z" },
      { "fake": "t" },
      { "key": "a" },
      { "key": "b" },
      { "key": "c" }
    ]);
    let expectation = v!([
      { "key": "a" },
      { "key": "b" },
      { "key": "c" },
      { "key": "X" },
      { "key": "Y" },
      { "key": "Z" },
      { "fake": "t" }
    ]);
    assert_eq!(expectation, filters!(sort_natural, input, v!("key")));
    assert_eq!(
        v!(["a", "b", "c", "X", "Y", "Z"]),
        filters!(sort_natural, v!(["X", "Y", "Z", "a", "b", "c"]))
    );
}

#[test]
#[ignore]
fn test_sort_empty_array() {
    assert_eq!(v!([]), filters!(sort, v!([]), v!("a")));
}

#[test]
#[ignore]
fn test_sort_natural_empty_array() {
    assert_eq!(v!([]), filters!(sort_natural, v!([]), v!("a")));
}

#[test]
#[ignore]
fn test_legacy_sort_hash() {
    assert_eq!(
        v!([{ "a": 1, "b": 2 }]),
        filters!(sort, v!({ "a": 1, "b": 2 }))
    );
}

#[test]
#[ignore]
fn test_numerical_vs_lexicographical_sort() {
    assert_eq!(v!([2, 10]), filters!(sort, v!([10, 2])));
    assert_eq!(
        v!([{ "a": 2 }, { "a": 10 }]),
        filters!(sort, v!([{ "a": 10 }, { "a": 2 }]), v!("a"))
    );
    assert_eq!(v!(["10", "2"]), filters!(sort, v!(["10", "2"])));
    assert_eq!(
        v!([{ "a": "10" }, { "a": "2" }]),
        filters!(sort, v!([{ "a": "10" }, { "a": "2" }]), v!("a"))
    );
}

#[test]
#[ignore]
fn test_uniq() {
    assert_eq!(v!(["foo"]), filters!(uniq, v!("foo")));
    assert_eq!(
        v!([1, 3, 2, 4]),
        filters!(uniq, v!([1, 1, 3, 2, 3, 1, 4, 3, 2, 1]))
    );
    assert_eq!(
        v!([{ "a": 1 }, { "a": 3 }, { "a": 2 }]),
        filters!(
            uniq,
            v!([{ "a": 1 }, { "a": 3 }, { "a": 1 }, { "a": 2 }]),
            v!("a")
        )
    );
    //testdrop: Implementation specific: Drops
}

#[test]
#[ignore]
fn test_uniq_empty_array() {
    assert_eq!(v!([]), filters!(uniq, v!([]), v!("a")));
}

#[test]
fn test_compact_empty_array() {
    assert_eq!(v!([]), filters!(compact, v!([]), v!("a")));
}

#[test]
fn test_reverse() {
    assert_eq!(v!([4, 3, 2, 1]), filters!(reverse, v!([1, 2, 3, 4])));
}

#[test]
#[ignore]
fn test_legacy_reverse_hash() {
    assert_eq!(
        v!([{ "a": 1, "b": 2 }]),
        filters!(reverse, v!({"a": 1, "b": 2}))
    );
}

#[test]
#[ignore]
fn test_map() {
    assert_eq!(
        v!([1, 2, 3, 4]),
        filters!(
            map,
            v!([{ "a": 1 }, { "a": 2 }, { "a": 3 }, { "a": 4 }]),
            v!("a")
        )
    );
    assert_template_result!(
        "abc",
        r#"{{ ary | map:"foo" | map:"bar" }}"#,
        v!({"ary": [{ "foo": { "bar": "a" } }, { "foo": { "bar": "b" } }, { "foo": { "bar": "c" } }]}),
    );
}

#[test]
#[should_panic]
fn test_map_doesnt_call_arbitrary_stuff() {
    panic!("Implementation specific: filters can't access arbitrary variables");
}

#[test]
#[should_panic]
fn test_map_calls_to_liquid() {
    panic!("Implementation specific: to_liquid");
}

#[test]
#[ignore]
fn test_map_on_hashes() {
    assert_template_result!(
        "4217",
        r#"{{ thing | map: "foo" | map: "bar" }}"#,
        v!({"thing": { "foo": [ { "bar": 42 }, { "bar": 17 } ] }}),
    );
}

#[test]
#[ignore]
fn test_legacy_map_on_hashes_with_dynamic_key() {
    let template = r#"{% assign key = "foo" %}{{ thing | map: key | map: "bar" }}"#;
    let hash = v!({ "foo": { "bar": 42 } });
    assert_template_result!("42", template, v!({ "thing": hash }));
}

#[test]
#[should_panic]
fn test_sort_calls_to_liquid() {
    panic!("Implementation specific: to_liquid");
}

#[test]
#[should_panic]
fn test_map_over_proc() {
    panic!("Implementation specific: proc");
}

#[test]
#[should_panic]
fn test_map_over_drops_returning_procs() {
    panic!("Implementation specific: proc / drops");
}

#[test]
#[should_panic]
fn test_map_works_on_enumerables() {
    panic!("Implementation specific: drops");
}

#[test]
#[should_panic]
fn test_sort_works_on_enumerables() {
    panic!("Implementation specific: drops");
}

#[test]
#[should_panic]
fn test_first_and_last_call_to_liquid() {
    panic!("Implementation specific: to_liquid");
}

#[test]
#[should_panic]
fn test_truncate_calls_to_liquid() {
    panic!("Implementation specific: to_liquid");
}

#[test]
#[ignore]
fn test_date() {
    assert_eq!(
        v!("May"),
        filters!(date, with_time("2006-05-05 10:00:00"), v!("%B"))
    );
    assert_eq!(
        v!("June"),
        filters!(date, with_time("2006-06-05 10:00:00"), v!("%B"))
    );
    assert_eq!(
        v!("July"),
        filters!(date, with_time("2006-07-05 10:00:00"), v!("%B"))
    );

    assert_eq!(
        v!("May"),
        filters!(date, v!("2006-05-05 10:00:00"), v!("%B"))
    );
    assert_eq!(
        v!("June"),
        filters!(date, v!("2006-06-05 10:00:00"), v!("%B"))
    );
    assert_eq!(
        v!("July"),
        filters!(date, v!("2006-07-05 10:00:00"), v!("%B"))
    );

    assert_eq!(
        v!("2006-07-05 10:00:00"),
        filters!(date, v!("2006-07-05 10:00:00"), v!(""))
    );
    assert_eq!(
        v!("2006-07-05 10:00:00"),
        filters!(date, v!("2006-07-05 10:00:00"), v!(""))
    );
    assert_eq!(
        v!("2006-07-05 10:00:00"),
        filters!(date, v!("2006-07-05 10:00:00"), v!(""))
    );
    assert_eq!(
        v!("2006-07-05 10:00:00"),
        filters!(date, v!("2006-07-05 10:00:00"), Nil)
    );

    assert_eq!(
        v!("07/05/2006"),
        filters!(date, v!("2006-07-05 10:00:00"), v!("%m/%d/%Y"))
    );

    assert_eq!(
        v!("07/16/2004"),
        filters!(date, v!("Fri Jul 16 01:00:00 2004"), v!("%m/%d/%Y"))
    );
    assert_eq!(
        v!("#{Date.today.year}"),
        filters!(date, v!("now"), v!("%Y"))
    );
    assert_eq!(
        v!("#{Date.today.year}"),
        filters!(date, v!("today"), v!("%Y"))
    );
    assert_eq!(
        v!("#{Date.today.year}"),
        filters!(date, v!("Today"), v!("%Y"))
    );

    assert_eq!(Nil, filters!(date, Nil, v!("%B")));

    assert_eq!(v!(""), filters!(date, v!(""), v!("%B")));

    // Limited in value because we can't change the timezone
    assert_eq!(
        v!("07/05/2006"),
        filters!(date, v!(1152098955), v!("%m/%d/%Y"))
    );
    assert_eq!(
        v!("07/05/2006"),
        filters!(date, v!("1152098955"), v!("%m/%d/%Y"))
    );
}

#[test]
#[ignore]
fn test_first_last() {
    assert_eq!(v!(1), filters!(first, v!([1, 2, 3])));
    assert_eq!(v!(3), filters!(last, v!([1, 2, 3])));
    assert_eq!(Nil, filters!(first, v!([])));
    assert_eq!(Nil, filters!(last, v!([])));
}

#[test]
fn test_replace() {
    assert_eq!(
        v!("2 2 2 2"),
        filters!(replace, v!("1 1 1 1"), v!("1"), v!(2))
    );
    assert_eq!(
        v!("2 2 2 2"),
        filters!(replace, v!("1 1 1 1"), v!(1), v!(2))
    );
    assert_eq!(
        v!("2 1 1 1"),
        filters!(replace_first, v!("1 1 1 1"), v!("1"), v!(2))
    );
    assert_eq!(
        v!("2 1 1 1"),
        filters!(replace_first, v!("1 1 1 1"), v!(1), v!(2))
    );
    assert_template_result!(
        "2 1 1 1",
        r#"{{ "1 1 1 1" | replace_first: "1", 2 }}"#,
        v!({}),
    );
}

#[test]
fn test_remove() {
    assert_eq!(v!("   "), filters!(remove, v!("a a a a"), v!("a")));
    assert_eq!(v!("   "), filters!(remove, v!("1 1 1 1"), v!(1)));
    assert_eq!(v!("a a a"), filters!(remove_first, v!("a a a a"), v!("a ")));
    assert_eq!(v!(" 1 1 1"), filters!(remove_first, v!("1 1 1 1"), v!(1)));
    assert_template_result!("a a a", r#"{{ "a a a a" | remove_first: "a " }}"#);
}

#[test]
fn test_pipes_in_string_arguments() {
    assert_template_result!("foobar", r#"{{ "foo|bar" | remove: "|" }}"#);
}

#[test]
fn test_strip() {
    assert_template_result!("ab c", "{{ source | strip }}", v!({"source": " ab c  "}));
    assert_template_result!(
        "ab c",
        "{{ source | strip }}",
        v!({"source": " \tab c  \n \t"}),
    );
}

#[test]
fn test_lstrip() {
    assert_template_result!("ab c  ", "{{ source | lstrip }}", v!({"source": " ab c  "}));
    assert_template_result!(
        "ab c  \n \t",
        "{{ source | lstrip }}",
        v!({"source": " \tab c  \n \t"}),
    );
}

#[test]
fn test_rstrip() {
    assert_template_result!(" ab c", "{{ source | rstrip }}", v!({"source": " ab c  "}));
    assert_template_result!(
        " \tab c",
        "{{ source | rstrip }}",
        v!({"source": " \tab c  \n \t"}),
    );
}

#[test]
fn test_strip_newlines() {
    assert_template_result!(
        "abc",
        "{{ source | strip_newlines }}",
        v!({"source": "a\nb\nc"}),
    );
    assert_template_result!(
        "abc",
        "{{ source | strip_newlines }}",
        v!({"source": "a\r\nb\nc"}),
    );
}

#[test]
#[ignore]
fn test_newlines_to_br() {
    assert_template_result!(
        "a<br />\nb<br />\nc",
        "{{ source | newline_to_br }}",
        v!({"source": "a\nb\nc"}),
    );
}

#[test]
#[ignore]
fn test_plus() {
    assert_template_result!("2", r#"{{ 1 | plus:1 }}"#);
    assert_template_result!("2.0", r#"{{ "1" | plus:"1.0" }}"#);

    // Implementation specific: use of drops
}

#[test]
#[ignore]
fn test_minus() {
    assert_template_result!(
        "4",
        r#"{{ input | minus:operand }}"#,
        v!({"input": 5, "operand": 1}),
    );
    assert_template_result!("2.3", r#"{{ "4.3" | minus:"2" }}"#);

    // Implementation specific: use of drops
}

#[test]
#[ignore]
fn test_abs() {
    assert_template_result!("17", r#"{{ 17 | abs }}"#);
    assert_template_result!("17", r#"{{ -17 | abs }}"#);
    assert_template_result!("17", r#"{{ "17" | abs }}"#);
    assert_template_result!("17", r#"{{ "-17" | abs }}"#);
    assert_template_result!("0", r#"{{ 0 | abs }}"#);
    assert_template_result!("0", r#"{{ "0" | abs }}"#);
    assert_template_result!("17.42", r#"{{ 17.42 | abs }}"#);
    assert_template_result!("17.42", r#"{{ -17.42 | abs }}"#);
    assert_template_result!("17.42", r#"{{ "17.42" | abs }}"#);
    assert_template_result!("17.42", r#"{{ "-17.42" | abs }}"#);
}

#[test]
#[ignore]
fn test_times() {
    assert_template_result!("12", r#"{{ 3 | times:4 }}"#);
    assert_template_result!("0", r#"{{ "foo" | times:4 }}"#);
    assert_template_result!(
        "6",
        r#"{{ "2.1" | times:3 | replace: ".","-" | plus:0}}"#,
        v!({}),
    );
    assert_template_result!("7.25", r#"{{ 0.0725 | times:100 }}"#);
    assert_template_result!("-7.25", r#"{{ "-0.0725" | times:100 }}"#);
    assert_template_result!("7.25", r#"{{ "-0.0725" | times: -100 }}"#);
    // Implementation specific: use of drops
}

#[test]
#[ignore]
fn test_divided_by() {
    assert_template_result!("4", r#"{{ 12 | divided_by:3 }}"#);
    assert_template_result!("4", r#"{{ 14 | divided_by:3 }}"#);

    assert_template_result!("5", r#"{{ 15 | divided_by:3 }}"#);
    assert_render_error!("{{ 5 | divided_by:0 }}");

    assert_template_result!("0.5", r#"{{ 2.0 | divided_by:4 }}"#);
    assert_render_error!("{{ 1 | modulo:0 }}");

    // Implementation specific: use of drops
}

#[test]
#[ignore]
fn test_modulo() {
    assert_template_result!("1", r#"{{ 3 | modulo:2 }}"#);
    assert_render_error!("{{ 1 | modulo:0 }}");

    // Implementation specific: use of drops
}

#[test]
#[ignore]
fn test_round() {
    assert_template_result!("5", r#"{{ input | round }}"#, v!({"input": 4.6}));
    assert_template_result!("4", r#"{{ "4.3" | round }}"#);
    assert_template_result!("4.56", r#"{{ input | round: 2 }}"#, v!({"input": 4.5612}));
    assert_render_error!("{{ 1.0 | divided_by: 0.0 | round }}");

    // Implementation specific: use of drops
}

#[test]
#[ignore]
fn test_ceil() {
    assert_template_result!("5", r#"{{ input | ceil }}"#, v!({"input": 4.6}));
    assert_template_result!("5", r#"{{ "4.3" | ceil }}"#);
    assert_render_error!("{{ 1.0 | divided_by: 0.0 | ceil }}");

    // Implementation specific: use of drops
}

#[test]
#[ignore]
fn test_floor() {
    assert_template_result!("4", r#"{{ input | floor }}"#, v!({"input": 4.6}));
    assert_template_result!("4", r#"{{ "4.3" | floor }}"#);
    assert_render_error!("{{ 1.0 | divided_by: 0.0 | floor }}");

    // Implementation specific: use of drops
}

#[test]
fn test_at_most() {
    assert_template_result!("4", r#"{{ 5 | at_most:4 }}"#);
    assert_template_result!("5", r#"{{ 5 | at_most:5 }}"#);
    assert_template_result!("5", r#"{{ 5 | at_most:6 }}"#);

    assert_template_result!("4.5", r#"{{ 4.5 | at_most:5 }}"#);
    // Implementation specific: use of drops
}

#[test]
fn test_at_least() {
    assert_template_result!("5", r#"{{ 5 | at_least:4 }}"#);
    assert_template_result!("5", r#"{{ 5 | at_least:5 }}"#);
    assert_template_result!("6", r#"{{ 5 | at_least:6 }}"#);

    assert_template_result!("5", r#"{{ 4.5 | at_least:5 }}"#);
    // Implementation specific: use of drops
}

#[test]
fn test_append() {
    let assigns = v!({ "a": "bc", "b": "d" });
    assert_template_result!("bcd", r#"{{ a | append: "d"}}"#, assigns.clone());
    assert_template_result!("bcd", r#"{{ a | append: b}}"#, assigns);
}

#[test]
fn test_concat() {
    assert_eq!(v!([1, 2, 3, 4]), filters!(concat, v!([1, 2]), v!([3, 4])));
    assert_eq!(v!([1, 2, "a"]), filters!(concat, v!([1, 2]), v!(["a"])));
    assert_eq!(v!([1, 2, 10]), filters!(concat, v!([1, 2]), v!([10])));

    filters_fail!(concat, v!([1, 2]), v!(10));
}

#[test]
fn test_prepend() {
    let assigns = v!({ "a": "bc", "b": "a" });
    assert_template_result!("abc", r#"{{ a | prepend: "a"}}"#, assigns.clone());
    assert_template_result!("abc", r#"{{ a | prepend: b}}"#, assigns);
}

#[test]
fn test_default() {
    assert_eq!(v!("foo"), filters!(default, v!("foo"), v!("bar")));
    assert_eq!(v!("bar"), filters!(default, Nil, v!("bar")));
    assert_eq!(v!("bar"), filters!(default, v!(""), v!("bar")));
    assert_eq!(v!("bar"), filters!(default, v!(false), v!("bar")));
    assert_eq!(v!("bar"), filters!(default, v!([]), v!("bar")));
    assert_eq!(v!("bar"), filters!(default, v!({}), v!("bar")));
}

#[test]
#[should_panic]
fn test_cannot_access_private_methods() {
    panic!("Implementation specific: filters can't access arbitrary variables");
}

#[test]
fn test_date_raises_nothing() {
    assert_template_result!("", r#"{{ "" | date: "%D" }}"#);
    assert_template_result!("abc", r#"{{ "abc" | date: "%D" }}"#);
}

#[test]
#[ignore]
fn test_where() {
    /*
    let input = v!([
      { "handle": "alpha", "ok": true },
      { "handle": "beta", "ok": false },
      { "handle": "gamma", "ok": false },
      { "handle": "delta", "ok": true }
    ]);

    let expectation = v!([
      { "handle": "alpha", "ok": true },
      { "handle": "delta", "ok": true }
    ]);

    assert_eq!(expectation, filters!(where, input, v!("ok"), true));
    assert_eq!(expectation, filters!(where, input, v!("ok")));
    */
}

#[test]
#[ignore]
fn test_where_no_key_set() {
    /*
    input = [
      { v!("handle"): v!("alpha"), v!("ok"): true },
      { v!("handle"): v!("beta") },
      { v!("handle"): v!("gamma") },
      { v!("handle"): v!("delta"), v!("ok"): true }
    ]

    expectation = [
      { v!("handle"): v!("alpha"), v!("ok"): true },
      { v!("handle"): v!("delta"), v!("ok"): true }
    ]

    assert_eq!(expectation, filters!(where, input, v!("ok"), true));
    assert_eq!(expectation, filters!(where, input, v!("ok")));
    */
}

#[test]
#[ignore]
fn test_where_non_array_map_input() {
    /*
    assert_eq!([{ v!("a"): v!("ok") }], filters!(where, { v!("a"): v!("ok") }, "a", r#"ok")#);
    assert_eq!([], filters!(where, { v!("a"): v!("not ok") }, "a", r#"ok")#);
    */
}

#[test]
#[ignore]
fn test_where_indexable_but_non_map_value() {
    /*
    assert_raises(Liquid::ArgumentError) { filters!(where, 1, v!("ok"), true) }
    assert_raises(Liquid::ArgumentError) { filters!(where, 1, v!("ok")) }
    */
}

#[test]
#[ignore]
fn test_where_non_boolean_value() {
    /*
    input = [
      { v!("message"): v!("Bonjour!"), v!("language"): v!("French") },
      { v!("message"): v!("Hello!"), v!("language"): v!("English") },
      { v!("message"): v!("Hallo!"), v!("language"): v!("German") }
    ]

    assert_eq!([{ v!("message"): v!("Bonjour!"), v!("language"): v!("French") }], filters!(where, input, "language", r#"French")#);
    assert_eq!([{ v!("message"): v!("Hallo!"), v!("language"): v!("German") }], filters!(where, input, "language", r#"German")#);
    assert_eq!([{ v!("message"): v!("Hello!"), v!("language"): v!("English") }], filters!(where, input, "language", r#"English")#);
    */
}

#[test]
#[ignore]
fn test_where_array_of_only_unindexable_values() {
    /*
    assert_eq!(Nil, filters!(where, [Nil], v!("ok"), true));
    assert_eq!(Nil, filters!(where, [Nil], v!("ok")));
    */
}

#[test]
#[ignore]
fn test_where_no_target_value() {
    /*
    input = [
      { v!("foo"): false },
      { v!("foo"): true },
      { v!("foo"): v!("for sure") },
      { v!("bar"): true }
    ]

    assert_eq!([{ v!("foo"): true }, { v!("foo"): v!("for sure") }], filters!(where, input, v!("foo")));
    */
}
