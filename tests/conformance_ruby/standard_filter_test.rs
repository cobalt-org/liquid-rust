use test_helper::*;

#[test]
fn test_size() {
    assert_eq!(v!(3), filters!(Size, v!([1, 2, 3])));
    assert_eq!(v!(0), filters!(Size, v!([])));
    assert_eq!(v!(0), filters!(Size, v!(nil)));
}

#[test]
fn test_downcase() {
    assert_eq!(v!("testing"), filters!(Downcase, v!("Testing")));
    assert_eq!(v!(""), filters!(Downcase, Nil));
}

#[test]
fn test_upcase() {
    assert_eq!(v!("TESTING"), filters!(Upcase, v!("Testing")));
    assert_eq!(v!(""), filters!(Upcase, Nil));
}

#[test]
#[should_panic] // liquid-rust#261
fn test_slice() {
    assert_eq!(v!("oob"), filters!(Slice, v!("foobar"), v!(1), v!(3)));
    assert_eq!(v!("oobar"), filters!(Slice, v!("foobar"), v!(1), v!(1000)));
    assert_eq!(v!(""), filters!(Slice, v!("foobar"), v!(1), v!(0)));
    assert_eq!(v!("o"), filters!(Slice, v!("foobar"), v!(1), v!(1)));
    assert_eq!(v!("bar"), filters!(Slice, v!("foobar"), v!(3), v!(3)));
    assert_eq!(v!("ar"), filters!(Slice, v!("foobar"), v!(-2), v!(2)));
    assert_eq!(v!("ar"), filters!(Slice, v!("foobar"), v!(-2), v!(1000)));
    assert_eq!(v!("r"), filters!(Slice, v!("foobar"), v!(-1)));
    assert_eq!(v!(""), filters!(Slice, Nil, v!(0)));
    assert_eq!(v!(""), filters!(Slice, v!("foobar"), v!(100), v!(10)));
    assert_eq!(v!(""), filters!(Slice, v!("foobar"), v!(-100), v!(10)));
    assert_eq!(v!("oob"), filters!(Slice, v!("foobar"), v!("1"), v!("3")));
    filters_fail!(Slice, v!("foobar"), Nil);
    filters_fail!(Slice, v!("foobar"), v!(0), v!(""));
}

#[test]
#[should_panic] // liquid-rust#261
fn test_slice_on_arrays() {
    let input = v!(["f", "o", "o", "b", "a", "r"]);
    assert_eq!(v!(["o", "o", "b"]), filters!(Slice, input, v!(1), v!(3)));
    assert_eq!(
        v!(["o", "o", "b", "a", "r"]),
        filters!(Slice, input, v!(1), v!(1000))
    );
    assert_eq!(v!([]), filters!(Slice, input, v!(1), v!(0)));
    assert_eq!(v!(["o"]), filters!(Slice, input, v!(1), v!(1)));
    assert_eq!(v!(["b", "a", "r"]), filters!(Slice, input, v!(3), v!(3)));
    assert_eq!(v!(["a", "r"]), filters!(Slice, input, v!(-2), v!(2)));
    assert_eq!(v!(["a", "r"]), filters!(Slice, input, v!(-2), v!(1000)));
    assert_eq!(v!(["r"]), filters!(Slice, input, v!(-1)));
    assert_eq!(v!([]), filters!(Slice, input, v!(100), v!(10)));
    assert_eq!(v!([]), filters!(Slice, input, v!(-100), v!(10)));
}

#[test]
#[should_panic] // liquid-rust#264
fn test_truncate() {
    assert_eq!(v!("1234..."), filters!(Truncate, v!("1234567890"), v!(7)));
    assert_eq!(
        v!("1234567890"),
        filters!(Truncate, v!("1234567890"), v!(20))
    );
    assert_eq!(v!("..."), filters!(Truncate, v!("1234567890"), v!(0)));
    assert_eq!(v!("1234567890"), filters!(Truncate, v!("1234567890")));
    assert_eq!(
        v!("测试..."),
        filters!(Truncate, v!("测试测试测试测试"), v!(5))
    );
    assert_eq!(
        v!("12341"),
        filters!(Truncate, v!("1234567890"), v!(5), v!(1))
    );
}

#[test]
#[should_panic] // liquid-rust#263
fn test_split() {
    assert_eq!(v!(["12", "34"]), filters!(Split, v!("12~34"), v!("~")));
    assert_eq!(
        v!(["A? ", " ,Z"]),
        filters!(Split, v!("A? ~ ~ ~ ,Z"), v!("~ ~ ~"))
    );
    assert_eq!(v!(["A?Z"]), filters!(Split, v!("A?Z"), v!("~")));
    assert_eq!(v!([]), filters!(Split, Nil, v!(" ")));
    assert_eq!(v!(["A", "Z"]), filters!(Split, v!("A1Z"), v!(1)));
}

#[test]
#[should_panic] // liquid-rust#253
fn test_escape() {
    assert_eq!(v!("&lt;strong&gt;"), filters!(Escape, v!("<strong>")));
    assert_eq!(v!("1"), filters!(Escape, v!(1)));
    assert_eq!(v!("2001-02-03"), filters!(Escape, date(2001, 2, 3)));
    assert_eq!(Nil, filters!(Escape, Nil));
}

#[test]
#[should_panic] // liquid-rust#269
fn test_h() {
    panic!("Implement this filter");
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
        filters!(EscapeOnce, v!("&lt;strong&gt;Hulk</strong>"))
    );
}

#[test]
#[should_panic] // liquid-rust#253
fn test_url_encode() {
    assert_eq!(
        v!("foo%2B1%40example.com"),
        filters!(UrlEncode, v!("foo+1@example.com"))
    );
    assert_eq!(v!("1"), filters!(UrlEncode, v!(1)));
    assert_eq!(v!("2001-02-03"), filters!(UrlEncode, date(2001, 2, 3)));
    assert_eq!(Nil, filters!(UrlEncode, Nil));
}

#[test]
#[should_panic] // liquid-rust#268
fn test_url_decode() {
    assert_eq!(v!("foo bar"), filters!(UrlDecode, v!("foo+bar")));
    assert_eq!(v!("foo bar"), filters!(UrlDecode, v!("foo%20bar")));
    assert_eq!(
        v!("foo+1@example.com"),
        filters!(UrlDecode, v!("foo%2B1%40example.com"))
    );
    assert_eq!(v!("1"), filters!(UrlDecode, v!(1)));
    assert_eq!(v!("2001-02-03"), filters!(UrlDecode, date(2001, 2, 3)));
    assert_eq!(Nil, filters!(UrlDecode, Nil));
}

#[test]
fn test_truncatewords() {
    assert_eq!(
        v!("one two three"),
        filters!(TruncateWords, v!("one two three"), v!(4))
    );
    assert_eq!(
        v!("one two..."),
        filters!(TruncateWords, v!("one two three"), v!(2))
    );
    assert_eq!(
        v!("one two three"),
        filters!(TruncateWords, v!("one two three"))
    );
    assert_eq!(v!("Two small (13&#8221; x 5.5&#8221; x 10&#8221; high) baskets fit inside one large basket (13&#8221;..."), filters!(TruncateWords, v!("Two small (13&#8221; x 5.5&#8221; x 10&#8221; high) baskets fit inside one large basket (13&#8221; x 16&#8221; x 10.5&#8221; high) with cover."), v!(15)));
    assert_eq!(
        v!("测试测试测试测试"),
        filters!(TruncateWords, v!("测试测试测试测试"), v!(5))
    );
    assert_eq!(
        v!("one two1"),
        filters!(TruncateWords, v!("one two three"), v!(2), v!(1))
    );
}

#[test]
fn test_strip_html() {
    assert_eq!(v!("test"), filters!(StripHtml, v!(r#"<div>test</div>"#)));
    assert_eq!(
        v!("test"),
        filters!(StripHtml, v!(r#"<div id="test">test</div>"#))
    );
    assert_eq!(
        v!(""),
        filters!(
            StripHtml,
            v!(r#"<script type="text/javascript">document.write"some stuff";</script>"#)
        )
    );
    assert_eq!(
        v!(""),
        filters!(StripHtml, v!(r#"<style type="text/css">foo bar</style>"#))
    );
    assert_eq!(
        v!("test"),
        filters!(StripHtml, v!(r#"<div\nclass="multiline">test</div>"#))
    );
    assert_eq!(
        v!("test"),
        filters!(StripHtml, v!(r#"<!-- foo bar \n test -->test"#))
    );
    assert_eq!(v!(""), filters!(StripHtml, Nil));
}

#[test]
fn test_join() {
    assert_eq!(v!("1 2 3 4"), filters!(Join, v!([1, 2, 3, 4])));
    assert_eq!(
        v!("1 - 2 - 3 - 4"),
        filters!(Join, v!([1, 2, 3, 4]), v!(" - "))
    );
    assert_eq!(v!("1121314"), filters!(Join, v!([1, 2, 3, 4]), v!(1)));
}

#[test]
#[should_panic] // liquid-rust#257
fn test_sort() {
    assert_eq!(v!([1, 2, 3, 4]), filters!(Sort, v!([4, 3, 2, 1])));
    assert_eq!(
        v!([{ "a": 1 }, { "a": 2 }, { "a": 3 }, { "a": 4 }]),
        filters!(
            Sort,
            v!([{ "a": 4 }, { "a": 3 }, { "a": 1 }, { "a": 2 }]),
            v!("a")
        )
    );
}

#[test]
#[should_panic] // liquid-rust#262
fn test_sort_with_nils() {
    assert_eq!(v!([1, 2, 3, 4, nil]), filters!(Sort, v!([nil, 4, 3, 2, 1])));
    assert_eq!(
        v!([{ "a": 1 }, { "a": 2 }, { "a": 3 }, { "a": 4 }, {}]),
        filters!(
            Sort,
            v!([{ "a": 4 }, { "a": 3 }, {}, { "a": 1 }, { "a": 2 }]),
            v!("a")
        )
    );
}

#[test]
#[should_panic] // liquid-rust#257
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
    assert_eq!(expectation, filters!(Sort, input, v!("price")));
}

#[test]
#[should_panic] // liquid-rust#257
fn test_sort_natural() {
    assert_eq!(
        v!(["a", "B", "c", "D"]),
        filters!(SortNatural, v!(["c", "D", "a", "B"]))
    );
    assert_eq!(
        v!([{ "a": "a" }, { "a": "B" }, { "a": "c" }, { "a": "D" }]),
        filters!(
            SortNatural,
            v!([{ "a": "D" }, { "a": "c" }, { "a": "a" }, { "a": "B" }]),
            v!("a")
        )
    );
}

#[test]
#[should_panic] // liquid-rust#262
fn test_sort_natural_with_nils() {
    assert_eq!(
        v!(["a", "B", "c", "D", nil]),
        filters!(SortNatural, v!([nil, "c", "D", "a", "B"]))
    );
    assert_eq!(
        v!([{ "a": "a" }, { "a": "B" }, { "a": "c" }, { "a": "D" }, {}]),
        filters!(
            SortNatural,
            v!([{ "a": "D" }, { "a": "c" }, {}, { "a": "a" }, { "a": "B" }]),
            v!("a")
        )
    );
}

#[test]
#[should_panic] // liquid-rust#257
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
    assert_eq!(expectation, filters!(SortNatural, input, v!("price")));
}

#[test]
#[should_panic] // liquid-rust#257
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
    assert_eq!(expectation, filters!(SortNatural, input, v!("key")));
    assert_eq!(
        v!(["a", "b", "c", "X", "Y", "Z"]),
        filters!(SortNatural, v!(["X", "Y", "Z", "a", "b", "c"]))
    );
}

#[test]
#[should_panic] // liquid-rust#257
fn test_sort_empty_array() {
    assert_eq!(v!([]), filters!(Sort, v!([]), v!("a")));
}

#[test]
#[should_panic] // liquid-rust#257
fn test_sort_natural_empty_array() {
    assert_eq!(v!([]), filters!(SortNatural, v!([]), v!("a")));
}

#[test]
#[should_panic] // liquid-rust#257
fn test_legacy_sort_hash() {
    assert_eq!(
        v!([{ "a": 1, "b": 2 }]),
        filters!(Sort, v!({ "a": 1, "b": 2 }))
    );
}

#[test]
#[should_panic] // liquid-rust#257
fn test_numerical_vs_lexicographical_sort() {
    assert_eq!(v!([2, 10]), filters!(Sort, v!([10, 2])));
    assert_eq!(
        v!([{ "a": 2 }, { "a": 10 }]),
        filters!(Sort, v!([{ "a": 10 }, { "a": 2 }]), v!("a"))
    );
    assert_eq!(v!(["10", "2"]), filters!(Sort, v!(["10", "2"])));
    assert_eq!(
        v!([{ "a": "10" }, { "a": "2" }]),
        filters!(Sort, v!([{ "a": "10" }, { "a": "2" }]), v!("a"))
    );
}

#[test]
#[should_panic] // liquid-rust#266
fn test_uniq() {
    assert_eq!(v!(["foo"]), filters!(Uniq, v!("foo")));
    assert_eq!(
        v!([1, 3, 2, 4]),
        filters!(Uniq, v!([1, 1, 3, 2, 3, 1, 4, 3, 2, 1]))
    );
    assert_eq!(
        v!([{ "a": 1 }, { "a": 3 }, { "a": 2 }]),
        filters!(
            Uniq,
            v!([{ "a": 1 }, { "a": 3 }, { "a": 1 }, { "a": 2 }]),
            v!("a")
        )
    );
    //testdrop: Implementation specific: Drops
}

#[test]
#[should_panic] // liquid-rust#267
fn test_uniq_empty_array() {
    assert_eq!(v!([]), filters!(Uniq, v!([]), v!("a")));
}

#[test]
#[should_panic] // liquid-rust#335
fn test_compact_empty_array() {
    assert_eq!(v!([]), filters!(Compact, v!([]), v!("a")));
}

#[test]
fn test_reverse() {
    assert_eq!(v!([4, 3, 2, 1]), filters!(Reverse, v!([1, 2, 3, 4])));
}

#[test]
#[should_panic] // liquid-rust#256
fn test_legacy_reverse_hash() {
    assert_eq!(
        v!([{ "a": 1, "b": 2 }]),
        filters!(Reverse, v!({"a": 1, "b": 2}))
    );
}

#[test]
fn test_map() {
    assert_eq!(
        v!([1, 2, 3, 4]),
        filters!(
            Map,
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
#[should_panic] // liquid-rust#255
fn test_map_on_hashes() {
    assert_template_result!(
        "4217",
        r#"{{ thing | map: "foo" | map: "bar" }}"#,
        v!({"thing": { "foo": [ { "bar": 42 }, { "bar": 17 } ] }}),
    );
}

#[test]
#[should_panic] // liquid-rust#255
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
#[should_panic] // liquid-rust#252
fn test_date() {
    assert_eq!(
        v!("May"),
        filters!(Date, with_time("2006-05-05 10:00:00"), v!("%B"))
    );
    assert_eq!(
        v!("June"),
        filters!(Date, with_time("2006-06-05 10:00:00"), v!("%B"))
    );
    assert_eq!(
        v!("July"),
        filters!(Date, with_time("2006-07-05 10:00:00"), v!("%B"))
    );

    assert_eq!(
        v!("May"),
        filters!(Date, v!("2006-05-05 10:00:00"), v!("%B"))
    );
    assert_eq!(
        v!("June"),
        filters!(Date, v!("2006-06-05 10:00:00"), v!("%B"))
    );
    assert_eq!(
        v!("July"),
        filters!(Date, v!("2006-07-05 10:00:00"), v!("%B"))
    );

    assert_eq!(
        v!("2006-07-05 10:00:00"),
        filters!(Date, v!("2006-07-05 10:00:00"), v!(""))
    );
    assert_eq!(
        v!("2006-07-05 10:00:00"),
        filters!(Date, v!("2006-07-05 10:00:00"), v!(""))
    );
    assert_eq!(
        v!("2006-07-05 10:00:00"),
        filters!(Date, v!("2006-07-05 10:00:00"), v!(""))
    );
    assert_eq!(
        v!("2006-07-05 10:00:00"),
        filters!(Date, v!("2006-07-05 10:00:00"), Nil)
    );

    assert_eq!(
        v!("07/05/2006"),
        filters!(Date, v!("2006-07-05 10:00:00"), v!("%m/%d/%Y"))
    );

    assert_eq!(
        v!("07/16/2004"),
        filters!(Date, v!("Fri Jul 16 01:00:00 2004"), v!("%m/%d/%Y"))
    );
    assert_eq!(
        v!("#{Date.today.year}"),
        filters!(Date, v!("now"), v!("%Y"))
    );
    assert_eq!(
        v!("#{Date.today.year}"),
        filters!(Date, v!("today"), v!("%Y"))
    );
    assert_eq!(
        v!("#{Date.today.year}"),
        filters!(Date, v!("Today"), v!("%Y"))
    );

    assert_eq!(Nil, filters!(Date, Nil, v!("%B")));

    assert_eq!(v!(""), filters!(Date, v!(""), v!("%B")));

    // Limited in value because we can't change the timezone
    assert_eq!(
        v!("07/05/2006"),
        filters!(Date, v!(1152098955), v!("%m/%d/%Y"))
    );
    assert_eq!(
        v!("07/05/2006"),
        filters!(Date, v!("1152098955"), v!("%m/%d/%Y"))
    );
}

#[test]
#[should_panic] // liquid-rust#254
fn test_first_last() {
    assert_eq!(v!(1), filters!(First, v!([1, 2, 3])));
    assert_eq!(v!(3), filters!(Last, v!([1, 2, 3])));
    assert_eq!(Nil, filters!(First, v!([])));
    assert_eq!(Nil, filters!(Last, v!([])));
}

#[test]
fn test_replace() {
    assert_eq!(
        v!("2 2 2 2"),
        filters!(Replace, v!("1 1 1 1"), v!("1"), v!(2))
    );
    assert_eq!(
        v!("2 2 2 2"),
        filters!(Replace, v!("1 1 1 1"), v!(1), v!(2))
    );
    assert_eq!(
        v!("2 1 1 1"),
        filters!(ReplaceFirst, v!("1 1 1 1"), v!("1"), v!(2))
    );
    assert_eq!(
        v!("2 1 1 1"),
        filters!(ReplaceFirst, v!("1 1 1 1"), v!(1), v!(2))
    );
    assert_template_result!(
        "2 1 1 1",
        r#"{{ "1 1 1 1" | replace_first: "1", 2 }}"#,
        v!({}),
    );
}

#[test]
fn test_remove() {
    assert_eq!(v!("   "), filters!(Remove, v!("a a a a"), v!("a")));
    assert_eq!(v!("   "), filters!(Remove, v!("1 1 1 1"), v!(1)));
    assert_eq!(v!("a a a"), filters!(RemoveFirst, v!("a a a a"), v!("a ")));
    assert_eq!(v!(" 1 1 1"), filters!(RemoveFirst, v!("1 1 1 1"), v!(1)));
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
fn test_newlines_to_br() {
    assert_template_result!(
        "a<br />\nb<br />\nc",
        "{{ source | newline_to_br }}",
        v!({"source": "a\nb\nc"}),
    );
}

#[test]
#[should_panic] // liquid-rust#260
fn test_plus() {
    assert_template_result!("2", r#"{{ 1 | plus:1 }}"#);
    assert_template_result!("2.0", r#"{{ "1" | plus:"1.0" }}"#);

    // Implementation specific: use of drops
}

#[test]
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
#[should_panic] // liquid-rust#265
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
#[should_panic] // liquid-rust#251
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
#[should_panic] // liquid-rust#251
fn test_modulo() {
    assert_template_result!("1", r#"{{ 3 | modulo:2 }}"#);
    assert_render_error!("{{ 1 | modulo:0 }}");

    // Implementation specific: use of drops
}

#[test]
#[should_panic] // liquid-rust#251
fn test_round() {
    assert_template_result!("5", r#"{{ input | round }}"#, v!({"input": 4.6}));
    assert_template_result!("4", r#"{{ "4.3" | round }}"#);
    assert_template_result!("4.56", r#"{{ input | round: 2 }}"#, v!({"input": 4.5612}));
    assert_render_error!("{{ 1.0 | divided_by: 0.0 | round }}");

    // Implementation specific: use of drops
}

#[test]
#[should_panic] // liquid-rust#251
fn test_ceil() {
    assert_template_result!("5", r#"{{ input | ceil }}"#, v!({"input": 4.6}));
    assert_template_result!("5", r#"{{ "4.3" | ceil }}"#);
    assert_render_error!("{{ 1.0 | divided_by: 0.0 | ceil }}");

    // Implementation specific: use of drops
}

#[test]
#[should_panic] // liquid-rust#251
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
    assert_eq!(v!([1, 2, 3, 4]), filters!(Concat, v!([1, 2]), v!([3, 4])));
    assert_eq!(v!([1, 2, "a"]), filters!(Concat, v!([1, 2]), v!(["a"])));
    assert_eq!(v!([1, 2, 10]), filters!(Concat, v!([1, 2]), v!([10])));

    filters_fail!(Concat, v!([1, 2]), v!(10));
}

#[test]
fn test_prepend() {
    let assigns = v!({ "a": "bc", "b": "a" });
    assert_template_result!("abc", r#"{{ a | prepend: "a"}}"#, assigns.clone());
    assert_template_result!("abc", r#"{{ a | prepend: b}}"#, assigns);
}

#[test]
fn test_default() {
    assert_eq!(v!("foo"), filters!(Default, v!("foo"), v!("bar")));
    assert_eq!(v!("bar"), filters!(Default, Nil, v!("bar")));
    assert_eq!(v!("bar"), filters!(Default, v!(""), v!("bar")));
    assert_eq!(v!("bar"), filters!(Default, v!(false), v!("bar")));
    assert_eq!(v!("bar"), filters!(Default, v!([]), v!("bar")));
    assert_eq!(v!("bar"), filters!(Default, v!({}), v!("bar")));
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
#[should_panic] // liquid-rust#291
fn test_where() {
    panic!("where is unimplemented");
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
#[should_panic] // liquid-rust#291
fn test_where_no_key_set() {
    panic!("where is unimplemented");
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
#[should_panic] // liquid-rust#291
fn test_where_non_array_map_input() {
    panic!("where is unimplemented");
    /*
    assert_eq!([{ v!("a"): v!("ok") }], filters!(where, { v!("a"): v!("ok") }, "a", r#"ok")#);
    assert_eq!([], filters!(where, { v!("a"): v!("not ok") }, "a", r#"ok")#);
    */
}

#[test]
#[should_panic] // liquid-rust#291
fn test_where_indexable_but_non_map_value() {
    panic!("where is unimplemented");
    /*
    assert_raises(Liquid::ArgumentError) { filters!(where, 1, v!("ok"), true) }
    assert_raises(Liquid::ArgumentError) { filters!(where, 1, v!("ok")) }
    */
}

#[test]
#[should_panic] // liquid-rust#291
fn test_where_non_boolean_value() {
    panic!("where is unimplemented");
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
#[should_panic] // liquid-rust#291
fn test_where_array_of_only_unindexable_values() {
    panic!("where is unimplemented");
    /*
    assert_eq!(Nil, filters!(where, [Nil], v!("ok"), true));
    assert_eq!(Nil, filters!(where, [Nil], v!("ok")));
    */
}

#[test]
#[should_panic] // liquid-rust#291
fn test_where_no_target_value() {
    panic!("where is unimplemented");
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
