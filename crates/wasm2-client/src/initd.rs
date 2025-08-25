#[cfg(feature = "console-backtrace")]
mod console_backtrace {
    use std::sync::LazyLock;

    use nill::Nil;

    pub static _NONE_: LazyLock<Nil> = LazyLock::new(console_error_panic_hook::set_once);
}
