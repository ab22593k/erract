//! Arena-based memory management for error context.

use smallvec::SmallVec;
use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Type alias for context storage.
pub type ContextVec = SmallVec<[(Cow<'static, str>, Cow<'static, str>); 1]>;

static NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(1);
thread_local! {
    static THREAD_ID: usize = NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed);
    static ARENA: RefCell<ContextArena> = RefCell::new(ContextArena::new());
}

/// Returns the current thread's unique ID.
pub fn current_thread_id() -> usize {
    THREAD_ID.with(|id| *id)
}

/// A handle to context stored either in an arena or on the heap.
#[derive(Debug, Clone, Default)]
pub enum ContextHandle {
    /// Context is stored in the thread-local arena.
    Arena {
        /// Offset in the arena's buffer.
        offset: usize,
        /// Number of context items.
        len: usize,
        /// Thread ID to ensure safety.
        thread_id: usize,
    },
    /// Context has been promoted to the heap (e.g., after cross-thread move or for large maps).
    Heap(Box<ContextVec>),
    /// No context attached.
    #[default]
    Empty,
}

#[cfg(feature = "serde")]
impl serde::Serialize for ContextHandle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let pairs = match self {
            Self::Empty => Vec::new(),
            Self::Heap(v) => v.to_vec(),
            Self::Arena {
                offset,
                len,
                thread_id,
            } => {
                if *thread_id == current_thread_id() {
                    with_arena(|arena| arena.get_pairs(*offset, *len))
                } else {
                    Vec::new()
                }
            }
        };
        let mut seq = serializer.serialize_seq(Some(pairs.len()))?;
        for pair in pairs {
            seq.serialize_element(&pair)?;
        }
        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ContextHandle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let pairs: Vec<(Cow<'static, str>, Cow<'static, str>)> =
            serde::Deserialize::deserialize(deserializer)?;
        if pairs.is_empty() {
            Ok(Self::Empty)
        } else {
            let mut v = ContextVec::new();
            for p in pairs {
                v.push(p);
            }
            Ok(Self::Heap(Box::new(v)))
        }
    }
}

impl ContextHandle {
    /// Returns the number of context items.
    pub fn len(&self) -> usize {
        match self {
            Self::Arena { len, .. } => *len,
            Self::Heap(v) => v.len(),
            Self::Empty => 0,
        }
    }

    /// Returns true if there is no context.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A thread-local arena for storing error context.
pub struct ContextArena {
    buffer: Vec<u8>,
}

impl ContextArena {
    fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(8192),
        }
    }

    /// Clears the arena. Should be called periodically to avoid memory growth.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Allocates space for context pairs in the arena.
    /// Returns the offset and length of the allocated block.
    pub fn push_pairs(
        &mut self,
        pairs: &[(Cow<'static, str>, Cow<'static, str>)],
    ) -> (usize, usize) {
        let offset = self.buffer.len();
        for (k, v) in pairs {
            let k_bytes = k.as_bytes();
            let v_bytes = v.as_bytes();
            self.buffer
                .extend_from_slice(&(k_bytes.len() as u32).to_le_bytes());
            self.buffer.extend_from_slice(k_bytes);
            self.buffer
                .extend_from_slice(&(v_bytes.len() as u32).to_le_bytes());
            self.buffer.extend_from_slice(v_bytes);
        }
        (offset, pairs.len())
    }

    /// Retrieves pairs from the arena at a given offset.
    pub fn get_pairs(
        &self,
        offset: usize,
        len: usize,
    ) -> Vec<(Cow<'static, str>, Cow<'static, str>)> {
        let mut pairs = Vec::with_capacity(len);
        let mut pos = offset;
        for _ in 0..len {
            if pos + 4 > self.buffer.len() {
                break;
            }
            let k_len = u32::from_le_bytes(self.buffer[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            if pos + k_len > self.buffer.len() {
                break;
            }
            let k = String::from_utf8_lossy(&self.buffer[pos..pos + k_len]).into_owned();
            pos += k_len;

            if pos + 4 > self.buffer.len() {
                break;
            }
            let v_len = u32::from_le_bytes(self.buffer[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            if pos + v_len > self.buffer.len() {
                break;
            }
            let v = String::from_utf8_lossy(&self.buffer[pos..pos + v_len]).into_owned();
            pos += v_len;

            pairs.push((Cow::Owned(k), Cow::Owned(v)));
        }
        pairs
    }
}

/// Gets the current thread's arena and executes a closure.
pub fn with_arena<R>(f: impl FnOnce(&mut ContextArena) -> R) -> R {
    ARENA.with(|arena| f(&mut arena.borrow_mut()))
}

/// Commits the current dynamic context to the arena.
pub fn commit_to_arena(pairs: &[(Cow<'static, str>, Cow<'static, str>)]) -> ContextHandle {
    let thread_id = current_thread_id();
    let (offset, len) = with_arena(|arena| arena.push_pairs(pairs));
    ContextHandle::Arena {
        offset,
        len,
        thread_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn test_large_context_spill() {
        let mut error = Error::not_found();
        for i in 0..100 {
            error = error.with_context(format!("key{i}"), format!("value{i}"));
        }

        assert_eq!(error.context().len(), 100);
    }

    #[test]
    fn test_error_stack_size() {
        let size = std::mem::size_of::<Error>();
        assert!(size <= 160, "Error size {size} exceeds 160 bytes");
    }

    #[test]
    fn test_context_handle_size() {
        let size = std::mem::size_of::<ContextHandle>();
        assert!(size <= 32, "ContextHandle size {size} exceeds 32 bytes");
    }

    #[test]
    fn test_arena_push_get() {
        let pairs = vec![
            (Cow::Borrowed("key1"), Cow::Borrowed("val1")),
            (Cow::Borrowed("key2"), Cow::Owned("val2".to_string())),
        ];

        with_arena(|arena| {
            let (offset, len) = arena.push_pairs(&pairs);
            let retrieved = arena.get_pairs(offset, len);
            assert_eq!(retrieved.len(), 2);
            assert_eq!(retrieved[0].0, "key1");
            assert_eq!(retrieved[1].1, "val2");
        });
    }

    #[test]
    fn test_arena_clear() {
        with_arena(|arena| {
            arena.push_pairs(&[(Cow::Borrowed("k"), Cow::Borrowed("v"))]);
            assert!(!arena.buffer.is_empty());
            arena.clear();
            assert_eq!(arena.buffer.len(), 0);
        });
    }
}
