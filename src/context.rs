use std::borrow::Cow;

use crate::Error;

/// Trait for adding context to errors.
///
/// This trait provides ergonomic methods for adding key-value context
/// to errors at the point of failure.
pub trait AddContext<T> {
    /// Adds a key-value pair to the error context.
    ///
    /// Both key and value are converted to `Cow<'static, str>`.
    /// Use static strings when possible for zero-copy performance.
    #[must_use]
    fn with_context(
        self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> T;

    /// Adds a key-value pair where value is converted via ToString.
    ///
    /// Use this when the value needs to be converted from a non-string type.
    #[must_use]
    fn with_context_value(self, key: impl Into<Cow<'static, str>>, value: impl ToString) -> T;
}

impl<T> AddContext<Result<T, Error>> for Result<T, Error> {
    #[inline]
    fn with_context(
        self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Result<T, Error> {
        self.map_err(|err| err.with_context(key, value))
    }

    #[inline]
    fn with_context_value(
        self,
        key: impl Into<Cow<'static, str>>,
        value: impl ToString,
    ) -> Result<T, Error> {
        self.map_err(|err| err.with_context_value(key, value))
    }
}

impl AddContext<Error> for Error {
    #[inline]
    fn with_context(
        self,
        key: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Error {
        self.with_context(key, value)
    }

    #[inline]
    fn with_context_value(self, key: impl Into<Cow<'static, str>>, value: impl ToString) -> Error {
        self.with_context_value(key, value)
    }
}

/// Extension trait for adding multiple context pairs at once.
pub trait AddContextIter {
    /// Adds multiple context key-value pairs.
    #[must_use]
    fn with_context_iter<K, V>(self, iter: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>;
}

impl<T> AddContextIter for Result<T, Error> {
    #[inline]
    fn with_context_iter<K, V>(self, iter: impl IntoIterator<Item = (K, V)>) -> Result<T, Error>
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        self.map_err(|err| err.with_context_iter(iter))
    }
}

impl AddContextIter for Error {
    #[inline]
    fn with_context_iter<K, V>(mut self, iter: impl IntoIterator<Item = (K, V)>) -> Error
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        for (key, value) in iter {
            self = self.with_context(key, value);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_context_on_err() {
        let result: Result<(), Error> =
            Err(Error::permanent(crate::ErrorKind::NotFound, "not found"));

        let result = result
            .with_context("user_id", "123")
            .with_context("operation", "lookup");

        if let Err(err) = result {
            assert_eq!(err.context().len(), 2);
            assert_eq!(err.context()[0].0, "user_id");
            assert_eq!(err.context()[0].1, "123");
        } else {
            panic!("expected error");
        }
    }

    #[test]
    fn test_with_context_value_on_err() {
        let result: Result<(), Error> =
            Err(Error::permanent(crate::ErrorKind::NotFound, "not found"));

        let result = result
            .with_context_value("user_id", 123)
            .with_context_value("count", 42u64);

        if let Err(err) = result {
            assert_eq!(err.context().len(), 2);
            assert_eq!(err.context()[0].0, "user_id");
            assert_eq!(err.context()[0].1, "123");
            assert_eq!(err.context()[1].1, "42");
        } else {
            panic!("expected error");
        }
    }

    #[test]
    fn test_with_context_on_ok() {
        let result: Result<(), Error> = Ok(());

        let result = result
            .with_context("user_id", "123")
            .with_context("operation", "lookup");

        assert!(result.is_ok());
    }

    #[test]
    fn test_with_context_iter() {
        let result: Result<(), Error> =
            Err(Error::permanent(crate::ErrorKind::NotFound, "not found"));

        let result = result.with_context_iter::<&str, &str>([
            ("user_id", "123"),
            ("operation", "lookup"),
            ("timestamp", "2024-01-01"),
        ]);

        if let Err(err) = result {
            assert_eq!(err.context().len(), 3);
        } else {
            panic!("expected error");
        }
    }
}
