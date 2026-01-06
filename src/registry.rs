//! Type-safe value registry for static persistence of literal values.
//!
//! This module provides a global registry that allows literal values to persist
//! across function calls within the same execution. Uses Arc for safe shared ownership.
//!
//! This registry is used by `Literal::__new` for testing. The `literal!` macro
//! uses per-call-site statics instead and doesn't need the registry.

use crate::inline::LiteralInner;
use crate::literal::Value;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Registry key: (file, line, column, type_id)
type RegistryKey = (PathBuf, u32, u32, TypeId);

/// Type-erased mutex wrapper stored in the registry
type RegistryValue = Arc<dyn Any + Send + Sync>;

/// Global registry mapping (file, line, column, type) to Arc<Mutex<LiteralInner<T>>>.
static VALUE_REGISTRY: Lazy<Mutex<HashMap<RegistryKey, RegistryValue>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Get or create a static literal value at the given source location.
///
/// This is used by `Literal::__new` for testing. The `literal!` macro uses
/// a per-call-site static instead.
#[doc(hidden)]
pub fn get_or_create<T: Value + Send + 'static>(
    initial: T,
    file: &str,
    line: u32,
    column: u32,
) -> Arc<Mutex<LiteralInner<T>>> {
    // Auto-start background flush thread on first literal access
    let _ = crate::flush::start_background_flush_internal();

    let path = PathBuf::from(file);
    let key = (path, line, column, TypeId::of::<LiteralInner<T>>());

    let mut registry = VALUE_REGISTRY.lock();

    if let Some(arc) = registry.get(&key) {
        // Downcast the Arc<dyn Any> to Arc<Mutex<LiteralInner<T>>>
        // This is safe because the TypeId in the key guarantees the correct type
        arc.clone()
            .downcast::<Mutex<LiteralInner<T>>>()
            .expect("TypeId mismatch in registry - this is a bug")
    } else {
        // Create a new entry
        let inner = LiteralInner::new(initial, file, line, column);
        let arc: Arc<Mutex<LiteralInner<T>>> = Arc::new(Mutex::new(inner));
        registry.insert(key, arc.clone());
        arc
    }
}
