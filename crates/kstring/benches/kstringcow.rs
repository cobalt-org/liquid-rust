#![feature(test)]

extern crate test;

use kstring::KStringCow;

mod macros;

mod kstringcow {
    use super::*;

    bench_clone_static!(bench_clone_static_00, 0, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_clone_static!(bench_clone_static_01, 1, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_clone_static!(bench_clone_static_02, 2, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_clone_static!(bench_clone_static_03, 3, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_clone_static!(bench_clone_static_04, 4, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_clone_static!(bench_clone_static_05, 5, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_clone_static!(bench_clone_static_06, 6, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_clone_static!(bench_clone_static_07, 7, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_clone_static!(bench_clone_static_08, 8, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_clone_static!(bench_clone_static_09, 9, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_clone_static!(
        bench_clone_static_10,
        10,
        |fixture| KStringCow::from_static(fixture)
    );
    bench_clone_static!(
        bench_clone_static_11,
        11,
        |fixture| KStringCow::from_static(fixture)
    );
    bench_clone_static!(
        bench_clone_static_12,
        12,
        |fixture| KStringCow::from_static(fixture)
    );
    bench_clone_static!(
        bench_clone_static_13,
        13,
        |fixture| KStringCow::from_static(fixture)
    );

    bench_clone_ref!(bench_clone_ref_00, 0, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_01, 1, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_02, 2, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_03, 3, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_04, 4, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_05, 5, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_06, 6, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_07, 7, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_08, 8, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_09, 9, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_10, 10, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_11, 11, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_12, 12, |fixture| KStringCow::from_ref(
        fixture
    ));
    bench_clone_ref!(bench_clone_ref_13, 13, |fixture| KStringCow::from_ref(
        fixture
    ));

    bench_clone_owned!(bench_clone_owned_00, 0, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_01, 1, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_02, 2, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_03, 3, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_04, 4, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_05, 5, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_06, 6, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_07, 7, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_08, 8, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_09, 9, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_10, 10, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_11, 11, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_12, 12, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_clone_owned!(bench_clone_owned_13, 13, |fixture| KStringCow::from_string(
        fixture
    ));

    bench_eq_static!(bench_eq_static_00, 0, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_01, 1, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_02, 2, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_03, 3, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_04, 4, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_05, 5, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_06, 6, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_07, 7, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_08, 8, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_09, 9, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_10, 10, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_11, 11, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_12, 12, |fixture| KStringCow::from_static(
        fixture
    ));
    bench_eq_static!(bench_eq_static_13, 13, |fixture| KStringCow::from_static(
        fixture
    ));

    bench_eq_ref!(bench_eq_ref_00, 0, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_01, 1, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_02, 2, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_03, 3, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_04, 4, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_05, 5, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_06, 6, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_07, 7, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_08, 8, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_09, 9, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_10, 10, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_11, 11, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_12, 12, |fixture| KStringCow::from_ref(fixture));
    bench_eq_ref!(bench_eq_ref_13, 13, |fixture| KStringCow::from_ref(fixture));

    bench_eq_owned!(bench_eq_owned_00, 0, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_01, 1, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_02, 2, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_03, 3, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_04, 4, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_05, 5, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_06, 6, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_07, 7, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_08, 8, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_09, 9, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_10, 10, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_11, 11, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_12, 12, |fixture| KStringCow::from_string(
        fixture
    ));
    bench_eq_owned!(bench_eq_owned_13, 13, |fixture| KStringCow::from_string(
        fixture
    ));
}
