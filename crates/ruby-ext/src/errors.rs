use magnus::{Error as MagnusError, Ruby};

pub(crate) fn runtime_error(ruby: &Ruby, message: impl Into<String>) -> MagnusError {
    MagnusError::new(ruby.exception_runtime_error(), message.into())
}

pub(crate) fn argument_error(ruby: &Ruby, message: impl Into<String>) -> MagnusError {
    MagnusError::new(ruby.exception_arg_error(), message.into())
}

pub(crate) fn syntax_error(ruby: &Ruby, message: impl Into<String>) -> MagnusError {
    runtime_error(ruby, message)
}
