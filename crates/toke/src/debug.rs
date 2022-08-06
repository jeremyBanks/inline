use {
    once_cell::sync::OnceCell,
    std::{fmt::Debug, sync::Weak},
};

#[macro_export(crate)]
macro_rules! fdebug {
    ($($tt:tt)*) => {
        $crate::debug::AsDebug(format!($($tt)*))
    };
}

#[macro_export(crate)]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

pub(crate) fn debug_weak<T: Debug>(weak: &Weak<T>) -> crate::debug::AsDebug {
    let t = std::any::type_name::<T>().rsplit("::").next().unwrap();
    if weak.strong_count() > 0 {
        let pointer = format!("{:?}", weak.as_ptr())[2..].to_ascii_uppercase();
        fdebug!("{t} at 0x{pointer}")
    } else if weak.ptr_eq(&Weak::new()) {
        fdebug!("empty Weak<{t}>")
    } else {
        fdebug!("dropped Weak<{t}>")
    }
}

pub(crate) fn debug_once_weak<T: Debug>(weak: &OnceCell<Weak<T>>) -> crate::debug::AsDebug {
    let t = std::any::type_name::<T>().rsplit("::").next().unwrap();
    if let Some(weak) = weak.get() {
        debug_weak(weak)
    } else {
        fdebug!("empty OnceCell<Weak<{t}>>")
    }
}

pub(crate) struct AsDebug<S: AsRef<str> = String>(S);
impl<S: AsRef<str>> core::fmt::Debug for AsDebug<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0.as_ref())
    }
}

pub(crate) fn short_string_debug(string: impl AsRef<str>) -> AsDebug<String> {
    let max_len = 16 + 2;

    let mut debug = format!("{:?}", string.as_ref());

    if debug.chars().count() <= max_len {
        return AsDebug(debug);
    }

    debug = debug.replace("\\n", " ");

    let folded = regex!(r"\s{2,}").replace_all(&debug, " ").to_string();

    if folded.len() <= max_len {
        return AsDebug(folded);
    } else if folded.len() < debug.len() {
        debug = folded;
    }

    let start_trimmed = "\"…".to_string() + debug[1..].trim_start();

    if start_trimmed.len() <= max_len {
        return AsDebug(start_trimmed);
    } else if start_trimmed.len() < debug.len() {
        debug = start_trimmed;
    }

    let end_trimmed = debug[..debug.len() - 1].trim_end().to_string() + "…\"";

    if end_trimmed.len() <= max_len {
        return AsDebug(end_trimmed);
    } else if end_trimmed.len() < debug.len() {
        debug = end_trimmed;
    }

    AsDebug(
        debug
            .chars()
            .take(max_len - 2)
            .chain(['…', '"'].into_iter())
            .collect(),
    )
}
