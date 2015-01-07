use filters::Size;

impl<'a> Fn<(&'a str,), String> for Size {
    extern "rust-call" fn call(&self, args: (&'a str,)) -> String {
        "".to_string()
    }
}
