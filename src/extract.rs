use crate::Error;
use exn::Frame;

/// Counts the number of error frames in the tree.
pub fn count_frames(exn: &exn::Exn<Error>) -> usize {
    walk_frame_count(exn.as_frame())
}

fn walk_frame_count(frame: &Frame) -> usize {
    let mut count = 1; // Current frame
    for child in frame.children() {
        count += walk_frame_count(child);
    }
    count
}

/// Gets the total number of errors in the tree (same as frame count).
pub fn count_errors(exn: &exn::Exn<Error>) -> usize {
    count_frames(exn)
}

/// Finds the first retryable error in the tree.
///
/// Returns `true` if any error in the tree is retryable.
pub fn has_retryable(exn: &exn::Exn<Error>) -> bool {
    walk_retryable(exn.as_frame())
}

fn walk_retryable(frame: &Frame) -> bool {
    // Check if the error in this frame is retryable
    if let Some(err_ref) = frame.as_any().downcast_ref::<Error>() {
        if err_ref.is_retryable() {
            return true;
        }
    }
    // Recurse into children
    for child in frame.children() {
        if walk_retryable(child) {
            return true;
        }
    }
    false
}

/// Finds the first permanent error in the tree.
///
/// Returns `true` if any error in the tree is permanent.
pub fn has_permanent(exn: &exn::Exn<Error>) -> bool {
    walk_permanent(exn.as_frame())
}

fn walk_permanent(frame: &Frame) -> bool {
    if let Some(err_ref) = frame.as_any().downcast_ref::<Error>() {
        if err_ref.is_permanent() {
            return true;
        }
    }
    for child in frame.children() {
        if walk_permanent(child) {
            return true;
        }
    }
    false
}

/// Checks if the error tree contains only retryable errors.
pub fn is_all_retryable(exn: &exn::Exn<Error>) -> bool {
    walk_all_retryable(exn.as_frame())
}

fn walk_all_retryable(frame: &Frame) -> bool {
    if let Some(err_ref) = frame.as_any().downcast_ref::<Error>() {
        if !err_ref.is_retryable() {
            return false;
        }
    }
    for child in frame.children() {
        if !walk_all_retryable(child) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorKind;
    use exn::{bail, ResultExt};

    #[test]
    fn test_count_frames() {
        fn inner() -> crate::Result<()> {
            bail!(Error::permanent(ErrorKind::NotFound, "inner"));
        }

        fn outer() -> crate::Result<()> {
            inner().or_raise(|| Error::temporary(ErrorKind::Unexpected, "outer"))?;
            Ok(())
        }

        let result = outer();
        if let Err(exn) = result {
            assert_eq!(count_frames(&exn), 2);
        }
    }

    #[test]
    fn test_count_errors() {
        fn inner() -> crate::Result<()> {
            bail!(Error::permanent(ErrorKind::NotFound, "inner"));
        }

        fn outer() -> crate::Result<()> {
            inner().or_raise(|| Error::temporary(ErrorKind::Unexpected, "outer"))?;
            Ok(())
        }

        let result = outer();
        if let Err(exn) = result {
            assert_eq!(count_errors(&exn), 2);
        }
    }

    #[test]
    fn test_has_retryable() {
        fn inner() -> crate::Result<()> {
            bail!(Error::temporary(ErrorKind::Timeout, "timeout"));
        }

        fn outer() -> crate::Result<()> {
            inner().or_raise(|| Error::permanent(ErrorKind::Unexpected, "outer"))?;
            Ok(())
        }

        let result = outer();
        if let Err(exn) = result {
            assert!(has_retryable(&exn));
        }
    }

    #[test]
    fn test_has_retryable_only_permanent() {
        fn inner() -> crate::Result<()> {
            bail!(Error::permanent(ErrorKind::NotFound, "not found"));
        }

        fn outer() -> crate::Result<()> {
            inner().or_raise(|| Error::permanent(ErrorKind::Unexpected, "outer"))?;
            Ok(())
        }

        let result = outer();
        if let Err(exn) = result {
            assert!(!has_retryable(&exn));
        }
    }

    #[test]
    fn test_has_permanent() {
        fn inner() -> crate::Result<()> {
            bail!(Error::permanent(ErrorKind::NotFound, "not found"));
        }

        fn outer() -> crate::Result<()> {
            inner().or_raise(|| Error::temporary(ErrorKind::Timeout, "outer"))?;
            Ok(())
        }

        let result = outer();
        if let Err(exn) = result {
            assert!(has_permanent(&exn));
        }
    }

    #[test]
    fn test_is_all_retryable() {
        fn inner() -> crate::Result<()> {
            bail!(Error::temporary(ErrorKind::Timeout, "timeout"));
        }

        fn outer() -> crate::Result<()> {
            inner().or_raise(|| Error::temporary(ErrorKind::Unexpected, "outer"))?;
            Ok(())
        }

        let result = outer();
        if let Err(exn) = result {
            assert!(is_all_retryable(&exn));
        }
    }

    #[test]
    fn test_not_all_retryable() {
        fn inner() -> crate::Result<()> {
            bail!(Error::permanent(ErrorKind::NotFound, "not found"));
        }

        fn outer() -> crate::Result<()> {
            inner().or_raise(|| Error::temporary(ErrorKind::Timeout, "outer"))?;
            Ok(())
        }

        let result = outer();
        if let Err(exn) = result {
            assert!(!is_all_retryable(&exn));
        }
    }
}
