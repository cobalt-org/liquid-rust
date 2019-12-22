pub static FIXTURES: &[&'static str] = &[
    "",
    "0",
    "01",
    "012",
    "0123",
    "01234",
    "012345",
    "0123456",
    "01234567",
    "012345678",
    "0123456789",
    "01234567890123456789",
    "0123456789012345678901234567890123456789",
    "01234567890123456789012345678901234567890123456789012345678901234567890123456789",
];

#[macro_export]
macro_rules! bench_clone_static {
    ($name:ident, $index:literal, $expr:expr) => {
        #[bench]
        fn $name(b: &mut test::Bencher) {
            let fixture = crate::macros::FIXTURES[$index];
            let uut = $expr(fixture);
            b.iter(|| uut.clone());
        }
    };
}

#[macro_export]
macro_rules! bench_clone_ref {
    ($name:ident, $index:literal, $expr:expr) => {
        #[bench]
        fn $name(b: &mut test::Bencher) {
            let fixture = String::from(crate::macros::FIXTURES[$index]);
            let uut = $expr(fixture.as_str());
            b.iter(|| uut.clone());
        }
    };
}

#[macro_export]
macro_rules! bench_clone_owned {
    ($name:ident, $index:literal, $expr:expr) => {
        #[bench]
        fn $name(b: &mut test::Bencher) {
            let fixture = String::from(crate::macros::FIXTURES[$index]);
            let uut = $expr(fixture);
            b.iter(|| uut.clone());
        }
    };
}

#[macro_export]
macro_rules! bench_eq_static {
    ($name:ident, $index:literal, $expr:expr) => {
        #[bench]
        fn $name(b: &mut test::Bencher) {
            let fixture = crate::macros::FIXTURES[$index];
            let uut = $expr(fixture);
            b.iter(|| uut == fixture);
        }
    };
}

#[macro_export]
macro_rules! bench_eq_ref {
    ($name:ident, $index:literal, $expr:expr) => {
        #[bench]
        fn $name(b: &mut test::Bencher) {
            let fixture = String::from(crate::macros::FIXTURES[$index]);
            let uut = $expr(fixture.as_str());
            b.iter(|| uut == fixture);
        }
    };
}

#[macro_export]
macro_rules! bench_eq_owned {
    ($name:ident, $index:literal, $expr:expr) => {
        #[bench]
        fn $name(b: &mut test::Bencher) {
            let fixture = String::from(crate::macros::FIXTURES[$index]);
            let uut = $expr(fixture.clone());
            b.iter(|| uut == fixture);
        }
    };
}
