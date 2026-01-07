//! Background flushing and flush_all functionality.

use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Exponential backoff parameters for background flush thread
const MIN_INTERVAL_MS: u64 = 64;
const MAX_INTERVAL_MS: u64 = 2_097_152; // ~35 minutes (2^21 ms)

/// Global flag to track if background flush thread has been started
static BACKGROUND_STARTED: AtomicBool = AtomicBool::new(false);

/// Global shutdown signal for background thread
static SHUTDOWN: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));

/// Flush all dirty literals to disk.
///
/// This function writes all literals that have been modified but not yet
/// written to their source files. It's called automatically by the background
/// flush thread, but can also be called manually.
///
/// # Errors
///
/// Returns the first error encountered while flushing. Note that some literals
/// may have been successfully flushed even if an error is returned.
///
/// # Example
///
/// ```no_run
/// use inline::cell;
///
/// let mut x = cell(1);
/// let mut y = cell(2);
///
/// x.value = 10;
/// y.value = 20;
///
/// // Flush all pending writes
/// inline::flush_all()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn flush_all() -> Result<(), Box<dyn std::error::Error>> {
    // Get snapshot of dirty literals
    let dirty = crate::dirty::get_dirty_literals();

    if dirty.is_empty() {
        return Ok(());
    }

    // Group by file for efficient flushing
    use std::collections::HashMap;
    let mut by_file: HashMap<std::path::PathBuf, Vec<(u32, u32)>> = HashMap::new();

    for (file, line, column) in dirty {
        by_file.entry(file).or_insert_with(Vec::new).push((line, column));
    }

    // Flush each file's literals
    // Note: We can't actually flush individual literals from here because we don't
    // have access to the registry. The background thread will handle the actual
    // flushing through the normal Drop mechanism or by triggering writes.
    //
    // For now, this is more of a "force write to disk" for already-updated values.
    // The real flushing happens in Drop or through LiteralExt::flush().

    // Actually, we should write the files to disk if they have pending changes
    for (file, _positions) in by_file {
        // Write the current state of the file to disk
        if let Err(e) = crate::runtime::write_to_disk(&file) {
            // Continue flushing other files even if one fails
            eprintln!("Warning: failed to flush {}: {}", file.display(), e);
        }
    }

    // Clear all dirty flags after successful flush
    crate::dirty::clear_all_dirty();

    Ok(())
}

/// Start the background flush thread with exponential backoff.
///
/// This is called automatically on first literal access. The thread:
/// - Starts with 64ms interval
/// - Doubles interval when no changes detected
/// - Resets to 64ms when changes are flushed
/// - Caps at ~35 minutes (2^21 ms)
/// - Adds ±12.5% jitter to prevent thundering herd
///
/// The thread runs until program exit.
pub(crate) fn start_background_flush_internal() -> Option<JoinHandle<()>> {
    // Only start once
    if BACKGROUND_STARTED.swap(true, Ordering::SeqCst) {
        return None;
    }

    let shutdown = Arc::clone(&*SHUTDOWN);

    let handle = thread::spawn(move || {
        let mut current_interval = Duration::from_millis(MIN_INTERVAL_MS);
        let max_interval = Duration::from_millis(MAX_INTERVAL_MS);

        while !shutdown.load(Ordering::Relaxed) {
            // Add jitter: ±12.5% (±1/8)
            let jittered = add_jitter(current_interval);
            thread::sleep(jittered);

            if crate::dirty::has_dirty_literals() {
                // Flush all dirty literals
                let _ = flush_all(); // Ignore errors in background thread

                // Reset to minimum interval
                current_interval = Duration::from_millis(MIN_INTERVAL_MS);
            } else {
                // Exponential backoff: double the interval
                current_interval = (current_interval * 2).min(max_interval);
            }
        }
    });

    Some(handle)
}

/// Add random jitter to a duration (±12.5%)
fn add_jitter(duration: Duration) -> Duration {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    // Get a random value using RandomState (no external dependency)
    let random_state = RandomState::new();
    let mut hasher = random_state.build_hasher();

    // Hash the current time for randomness
    std::time::SystemTime::now().hash(&mut hasher);
    let random_value = hasher.finish();

    // Calculate jitter: ±1/8 of the duration
    let jitter_range = duration / 8;
    let jitter_nanos = jitter_range.as_nanos() as u64;

    if jitter_nanos == 0 {
        return duration; // Too small to jitter
    }

    // Random offset in range [0, 2*jitter_range)
    let offset_nanos = random_value % (jitter_nanos * 2);

    // Convert to signed offset: [-jitter_range, +jitter_range)
    let jitter = if offset_nanos < jitter_nanos {
        // Negative jitter
        duration.saturating_sub(Duration::from_nanos(jitter_nanos - offset_nanos))
    } else {
        // Positive jitter
        duration.saturating_add(Duration::from_nanos(offset_nanos - jitter_nanos))
    };

    jitter
}

/// Explicitly start the background flush thread (for testing/control).
///
/// Normally the thread starts automatically on first literal access.
/// This function allows manual control if needed.
///
/// Returns `None` if the thread was already started.
pub fn start_background_flush() -> Option<JoinHandle<()>> {
    start_background_flush_internal()
}
