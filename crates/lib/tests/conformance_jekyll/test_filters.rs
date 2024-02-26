use liquid_lib::jekyll;

mod sort_filter {
    use super::*;

    #[test]
    fn raise_exception_when_input_is_nil() {
        let input = liquid_core::Value::Nil;
        assert!(liquid_core::call_filter!(jekyll::Sort, input).is_err());
    }
    #[test]
    fn return_sorted_numbers() {
        assert_eq!(
            v!([1, 2, 2.2, 3]),
            liquid_core::call_filter!(jekyll::Sort, v!([3, 2.2, 2, 1])).unwrap()
        );
    }

    #[test]
    fn return_sorted_strings() {
        assert_eq!(
            v!(["10", "2"]),
            liquid_core::call_filter!(jekyll::Sort, v!(["10", "2"])).unwrap()
        );
        assert_eq!(
            v!(["FOO", "Foo", "foo"]),
            liquid_core::call_filter!(jekyll::Sort, v!(["foo", "Foo", "FOO"])).unwrap()
        );
        assert_eq!(
            v!(["_foo", "foo", "foo_"]),
            liquid_core::call_filter!(jekyll::Sort, v!(["foo_", "_foo", "foo"])).unwrap()
        );
        // Cyrillic
        assert_eq!(
            v!(["ВУЗ", "Вуз", "вуз"]),
            liquid_core::call_filter!(jekyll::Sort, v!(["Вуз", "вуз", "ВУЗ"])).unwrap()
        );
        assert_eq!(
            v!(["_вуз", "вуз", "вуз_"]),
            liquid_core::call_filter!(jekyll::Sort, v!(["вуз_", "_вуз", "вуз"])).unwrap()
        );
        // Hebrew
        assert_eq!(
            v!(["אלף", "בית"]),
            liquid_core::call_filter!(jekyll::Sort, v!(["בית", "אלף"])).unwrap()
        );
    }

    /*
    #[test]
    fn return_sorted_by_property_array() {
      assert_eq!( liquid_core::value!([{ "a" => 1 }, { "a" => 2 }, { "a" => 3 }, { "a" => 4 }]),
                   liquid_core::call_filter!(liquid_core::value!([{ "a" => 4 }, { "a" => 3 }, { "a" => 1 }, { "a" => 2 }], "a")));
    }

    #[test]
    fn return_sorted_by_property_array_with_numeric_strings_sorted_as_numbers() {
      assert_eq!(([{ "a" => ".5" }, { "a" => "0.65" }, { "a" => "10" }],
                   liquid_core::call_filter!([{ "a" => "10" }, { "a" => ".5" }, { "a" => "0.65" }], "a")));
    }

    #[test]
    fn return_sorted_by_property_array_with_numeric_strings_first() {
      assert_eq!(([{ "a" => ".5" }, { "a" => "0.6" }, { "a" => "twelve" }],
                   liquid_core::call_filter!([{ "a" => "twelve" }, { "a" => ".5" }, { "a" => "0.6" }], "a")));
    }

    #[test]
    fn return_sorted_by_property_array_with_numbers_and_strings () {
      assert_eq!(([{ "a" => "1" }, { "a" => "1abc" }, { "a" => "20" }],
                   liquid_core::call_filter!([{ "a" => "20" }, { "a" => "1" }, { "a" => "1abc" }], "a")));
    }

    #[test]
    fn return_sorted_by_property_array_with_nils_first() {
      ary = [{ "a" => 2 }, { "b" => 1 }, { "a" => 1 }]
      assert_eq!( [{ "b" => 1 }, { "a" => 1 }, { "a" => 2 }], liquid_core::call_filter!(ary, "a"));
      assert_eq!( liquid_core::call_filter!(ary, "a"), liquid_core::call_filter!(ary, "a", "first"));
    }

    #[test]
    fn return_sorted_by_property_array_with_nils_last() {
      assert_eq!( [{ "a" => 1 }, { "a" => 2 }, { "b" => 1 }],
                   liquid_core::call_filter!([{ "a" => 2 }, { "b" => 1 }, { "a" => 1 }], "a", "last"));
    }

    #[test]
    fn return_sorted_by_subproperty_array() {
      assert_eq!( [{ "a" => { "b" => 1 } }, { "a" => { "b" => 2 } },
                    { "a" => { "b" => 3 } },],
                   liquid_core::call_filter!([{ "a" => { "b" => 2 } }, { "a" => { "b" => 1 } },
                                 { "a" => { "b" => 3 } },], "a.b"));
    }
    */
}
