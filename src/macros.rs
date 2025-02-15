//! Internal macros

/// Unwraps using [`std::panic::panic_any`], needed to panic with any structure in Rust 2021.
/// See [https://github.com/rust-lang/rust/issues/78500]
#[macro_export]
macro_rules! unwrap_any {
    ($pyfunc:expr) => {
        $pyfunc.unwrap_or_else(|err| std::panic::panic_any(err))
    };
}

#[macro_export]
/// Creates a new scope for the expression where the `?` operator can be used if the expression
/// returns the same type of the `?` call, then unwraps the result using [`std::panic::panic_any`]
/// This can be used to panic and send a proper error message to the python interpreter.
///
/// ```rust
/// // Example usage
///
/// use std::num::ParseIntError;
/// use bytewax::try_unwrap;
///
/// struct RangeParser {
///     start: u64,
///     end: Option<u64>,
/// }
///
/// impl RangeParser {
///     pub fn new(start: &str) -> Result<RangeParser, ParseIntError> {
///         let start = start.parse::<u64>()?;
///         Ok(Self { start, end: None })
///     }
///
///     pub fn end(mut self, end: &str) -> Result<RangeParser, ParseIntError> {
///         self.end = Some(end.parse::<u64>()?);
///         Ok(self)
///     }
/// }
///
/// let res = try_unwrap!(RangeParser::new("0")?.end("12"));
/// assert_eq!(res.start, 0);
/// assert_eq!(res.end, Some(12));
/// ```
macro_rules! try_unwrap {
    ($pyfunc:expr) => {
        // This would be the perfect use for the
        // https://doc.rust-lang.org/nightly/unstable-book/language-features/try-blocks.html
        // feature.
        (|| $pyfunc)().unwrap_or_else(|err| std::panic::panic_any(err))
    };
}

#[macro_export]
macro_rules! log_func {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

#[macro_export]
/// This macro generates some boilerplate for classes exposed to Python.
/// This is needed mainly for pickling and unpickling of the objects.
///
/// ```rust
/// // Example usage:
/// use bytewax::add_pymethods;
/// use chrono::Duration;
/// use pyo3::{pyclass, Python};
///
/// #[pyclass(module = "bytewax.window", subclass)]
/// #[pyo3(text_signature = "()")]
/// struct WindowConfig;
///
/// #[pyclass(module="bytewax.config", extends=WindowConfig)]
/// #[derive(Clone)]
/// struct SlidingWindow { length: Duration };
///
/// add_pymethods!(
///     SlidingWindow,
///     parent: WindowConfig,
///     signature: (length),
///     args {
///         length: Duration => Duration::zero()
///     }
/// );
/// ```
macro_rules! add_pymethods {(
    $struct:ident,
    parent: $parent:ident,
    signature: $signature:tt,
    args { $($arg:ident: $arg_type:ty => $default:expr),* }
) => {
    #[pyo3::pymethods]
    impl $struct {
        #[new]
        #[pyo3(signature=$signature)]
        pub(crate) fn py_new($($arg: $arg_type),*) -> (Self, $parent) {
            (Self { $($arg),* }, $parent {})
        }

        /// Return a representation of this class as a PyDict.
        fn __getstate__(&self) -> std::collections::HashMap<&str, pyo3::Py<pyo3::PyAny>> {
            pyo3::Python::with_gil(|py| {
                std::collections::HashMap::from([
                    ("type", pyo3::IntoPy::into_py(stringify!($struct), py)),
                    $((stringify!($arg), pyo3::IntoPy::into_py(self.$arg.clone(), py))),*
                ])
            })
        }

        /// Egregious hack because pickling assumes the type has "empty"
        /// mutable objects.
        ///
        /// Pickle always calls `__new__(*__getnewargs__())` but notice we
        /// don't have access to the pickled `db_file_path` yet, so we
        /// have to pass in some dummy string value that will be
        /// overwritten by `__setstate__()` shortly.
        #[allow(unused_parens)]
        fn __getnewargs__(&self) -> ($($arg_type,) *) {
            ($($default,) *)
        }

        /// Unpickle from a PyDict
        fn __setstate__(&mut self, state: &pyo3::PyAny) -> pyo3::PyResult<()> {
            #[allow(unused_variables)]
            let dict: &pyo3::types::PyDict = state.downcast()?;
            // This is like crate::common::pickle_extract
            // Duplicated here so that we can doctest this macro
            // without making `pickle_extract` public.
            $(
            self.$arg = dict
                .get_item(stringify!($arg))
                .ok_or_else(|| pyo3::exceptions::PyValueError::new_err(
                    format!("bad pickle contents for {}: {}", stringify!($arg), dict)
                ))?
                .extract()?;
            )*
            Ok(())
        }
    }
}}
