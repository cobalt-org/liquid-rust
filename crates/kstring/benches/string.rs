#![feature(test)]

extern crate test;

mod macros;

mod string {
    use super::*;

    bench_clone_static!(bench_clone_static_00, 0, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_01, 1, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_02, 2, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_03, 3, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_04, 4, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_05, 5, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_06, 6, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_07, 7, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_08, 8, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_09, 9, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_10, 10, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_11, 11, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_12, 12, |fixture| String::from(fixture));
    bench_clone_static!(bench_clone_static_13, 13, |fixture| String::from(fixture));

    bench_clone_owned!(bench_clone_owned_00, 0, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_01, 1, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_02, 2, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_03, 3, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_04, 4, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_05, 5, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_06, 6, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_07, 7, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_08, 8, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_09, 9, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_10, 10, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_11, 11, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_12, 12, |fixture| String::from(fixture));
    bench_clone_owned!(bench_clone_owned_13, 13, |fixture| String::from(fixture));

    bench_eq_static!(bench_eq_static_00, 0, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_01, 1, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_02, 2, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_03, 3, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_04, 4, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_05, 5, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_06, 6, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_07, 7, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_08, 8, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_09, 9, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_10, 10, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_11, 11, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_12, 12, |fixture| String::from(fixture));
    bench_eq_static!(bench_eq_static_13, 13, |fixture| String::from(fixture));

    bench_eq_owned!(bench_eq_owned_00, 0, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_01, 1, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_02, 2, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_03, 3, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_04, 4, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_05, 5, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_06, 6, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_07, 7, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_08, 8, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_09, 9, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_10, 10, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_11, 11, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_12, 12, |fixture| String::from(fixture));
    bench_eq_owned!(bench_eq_owned_13, 13, |fixture| String::from(fixture));
}
