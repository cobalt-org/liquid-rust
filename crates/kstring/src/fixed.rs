use std::fmt;

macro_rules! fixed_string {
    ($name:ident, $len:literal) => {
        #[derive(Clone)]
        pub(crate) struct $name {
            array: [u8; $len],
        }

        impl $name {
            pub(crate) fn new(s: &str) -> Self {
                assert_eq!(s.as_bytes().len(), $len);
                let mut array = [0; $len];
                array.copy_from_slice(&s.as_bytes()[0..$len]);
                Self { array }
            }

            #[inline]
            pub(crate) fn to_boxed_str(&self) -> Box<str> {
                Box::from(self.as_str())
            }

            #[inline]
            pub(crate) fn as_str(&self) -> &str {
                unsafe { std::str::from_utf8_unchecked(&self.array) }
            }
        }

        impl fmt::Debug for $name {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(self.as_str(), f)
            }
        }
    };
}

fixed_string!(FixedString1, 1);
fixed_string!(FixedString2, 2);
fixed_string!(FixedString3, 3);
fixed_string!(FixedString4, 4);
fixed_string!(FixedString5, 5);
fixed_string!(FixedString6, 6);
fixed_string!(FixedString7, 7);
fixed_string!(FixedString8, 8);
fixed_string!(FixedString9, 9);
fixed_string!(FixedString10, 10);
fixed_string!(FixedString11, 11);
fixed_string!(FixedString12, 12);
fixed_string!(FixedString13, 13);
fixed_string!(FixedString14, 14);
fixed_string!(FixedString15, 15);
fixed_string!(FixedString16, 16);
