use crate::Error;
use exn::Frame;
use smallvec::SmallVec;

/// An iterator that traverses the error frame tree in depth-first order.
///
/// This iterator uses an inline stack (SmallVec) to avoid heap allocation
/// for error trees with moderate depth (up to 16 levels).
struct FrameIter<'a> {
    // 16 frames is enough for most practical error chains.
    // If deeper, it spills to the heap automatically.
    stack: SmallVec<[&'a Frame; 16]>,
}

impl<'a> FrameIter<'a> {
    fn new(root: &'a Frame) -> Self {
        let mut stack = SmallVec::new();
        stack.push(root);
        Self { stack }
    }
}

impl<'a> Iterator for FrameIter<'a> {
    type Item = &'a Frame;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.stack.pop()?;

        // Add children to the stack.
        // Note: This results in traversing children in reverse order (last child first).
        // Since we are only checking boolean properties or counting, strict order
        // usually doesn't matter for these operations.
        for child in frame.children() {
            self.stack.push(child);
        }

        Some(frame)
    }
}

/// Counts the number of error frames in the tree.
///
/// This operation is iterative and safe for deep error trees.
pub fn count_frames(exn: &exn::Exn<Error>) -> usize {
    FrameIter::new(exn.as_frame()).count()
}

/// Gets the total number of errors in the tree (same as frame count).
pub fn count_errors(exn: &exn::Exn<Error>) -> usize {
    count_frames(exn)
}

/// Finds the first retryable error in the tree.
///
/// Returns `true` if any error in the tree is retryable.
/// This operation is iterative and safe for deep error trees.
pub fn has_retryable(exn: &exn::Exn<Error>) -> bool {
    FrameIter::new(exn.as_frame()).any(|frame| {
        frame
            .as_any()
            .downcast_ref::<Error>()
            .is_some_and(|e| e.is_retryable())
    })
}

/// Finds the first permanent error in the tree.
///
/// Returns `true` if any error in the tree is permanent.
/// This operation is iterative and safe for deep error trees.
pub fn has_permanent(exn: &exn::Exn<Error>) -> bool {
    FrameIter::new(exn.as_frame()).any(|frame| {
        frame
            .as_any()
            .downcast_ref::<Error>()
            .is_some_and(|e| e.is_permanent())
    })
}

/// Checks if the error tree contains only retryable errors.
///
/// This operation is iterative and safe for deep error trees.
pub fn is_all_retryable(exn: &exn::Exn<Error>) -> bool {
    FrameIter::new(exn.as_frame()).all(|frame| {
        frame
            .as_any()
            .downcast_ref::<Error>()
            .is_none_or(|e| e.is_retryable())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorKind;
    use exn::{ResultExt, bail};

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
    fn test_deep_recursion_safety() {
        // Create a deep error tree (1000 levels) to verify no stack overflow
        let mut result: crate::Result<()> =
            Err(Error::permanent(ErrorKind::NotFound, "base").raise());

        for i in 0..1000 {
            // We need to move the result to wrap it
            result =
                result.or_raise(|| Error::temporary(ErrorKind::Unexpected, format!("wrap {i}")));
        }

        if let Err(exn) = result {
            // This would stack overflow with recursive implementation
            assert_eq!(count_frames(&exn), 1001);
            assert!(has_retryable(&exn));
            assert!(has_permanent(&exn));
        } else {
            panic!("expected error");
        }
    }
}
