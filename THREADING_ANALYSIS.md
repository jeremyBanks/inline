# Thread Safety Analysis

## Architecture

The system uses a three-tier synchronization model:

1. **Global state**: `FILE_STATES` - `RwLock<HashMap<PathBuf, FileState>>`
2. **Shared state per file**: `FileState::shared` - `Arc<RwLock<SharedState>>`
3. **Thread-local cache**: Each thread has `RefCell<HashMap<PathBuf, CachedState>>`

## Race Condition Analysis

### ✅ Scenario 1: Concurrent Updates to Different Macros

**Timeline:**
```
T1: litter1.set(100)  // macro at index 0
T2: litter2.set(200)  // macro at index 1

1. T1 acquires write lock in update_macro_by_index()
2. T2 blocks waiting for write lock
3. T1 does char-range splicing, updates shared.source, increments version
4. T1 releases write lock
5. T2 acquires write lock
6. T2 reads shared.source (gets T1's updated version!)
7. T2 does char-range splicing on T1's updated source
8. Both updates are preserved correctly
```

**Safe because:** Write lock on `SharedState` serializes all modifications.

### ✅ Scenario 2: Read During Write

**Timeline:**
```
T1: litter1.set(100)
T2: litter2.get()

1. T1 acquires write lock
2. T2 calls get_cached_ast(), tries to acquire read lock
3. T2 blocks until T1 releases write lock
4. T2 sees consistent state
```

**Safe because:** `RwLock` ensures readers don't see partial updates.

### ✅ Scenario 3: Lazy Initialization Race

**Code** (lines 614-636):
```rust
fn get_or_load_file_state(path: &Path) -> Result<FileState, io::Error> {
    let states = FILE_STATES.read();
    if let Some(state) = states.get(path) {
        return Ok(state.clone());  // Fast path
    }
    drop(states);  // Release read lock

    let mut states = FILE_STATES.write();
    // Double-check in case another thread loaded it
    if let Some(state) = states.get(path) {
        return Ok(state.clone());
    }

    let state = FileState::load(path)?;
    states.insert(path.to_path_buf(), state.clone());
    Ok(state)
}
```

**Safe because:** Classic double-check locking pattern prevents duplicate loads.

### ✅ Scenario 4: Mutable Access to Same Inline Instance

**Prevented by Rust's borrow checker:**
```rust
impl<T: Literal> Inline<T> {
    pub fn set(&mut self, new_value: T) { ... }
}
```

Since `set()` requires `&mut self`, the borrow checker prevents concurrent mutable access.
You'd need `Arc<Mutex<Inline<T>>>` to share across threads, which provides synchronization.

### ✅ Scenario 5: External File Modification

**Detection in `write_to_disk`** (lines 478-512):
```rust
pub fn write_to_disk(&self) -> Result<(), io::Error> {
    let current_disk_content = fs::read_to_string(&self.path)?;

    let mut shared = self.shared.write();

    if current_disk_content != shared.disk_source {
        panic!("CONCURRENT MODIFICATION DETECTED!");
    }

    fs::write(&self.path, &shared.source)?;
    shared.disk_source = shared.source.clone();
    Ok(())
}
```

**Safe because:**
- `disk_source` tracks what we last wrote/read
- Any external changes are detected and rejected
- Prevents data loss from concurrent external modifications

### ✅ Scenario 6: Thread-Local Cache Staleness

**Version-based invalidation** (lines 151-190):
```rust
fn get_cached_ast(&self) -> Result<...> {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();

        let shared = self.shared.read();
        let current_version = shared.version;
        drop(shared);  // Release lock immediately

        if let Some(cached) = cache.get(&self.path) {
            if cached.version == current_version {
                return Ok((cached.ast.clone(), ...));  // Cache hit
            }
        }

        // Cache miss - re-parse with current source
        // ...
    })
}
```

**Safe because:**
- Each modification increments `shared.version`
- Thread-local caches check version before using cached AST
- Stale caches are automatically invalidated

### ✅ Scenario 7: Critical Section Atomicity

**Character-range splicing** (lines 297-334):
```rust
pub fn update_macro_by_index(...) -> Result<(), String> {
    let mut shared = self.shared.write();  // ACQUIRE WRITE LOCK

    let source = shared.source.clone();
    let ast = syn::parse_file(&source)?;
    let value_span = Self::find_macro_value_span_static(&ast, index, &source)?;

    // Character-range splicing
    let mut new_source = String::with_capacity(source.len());
    new_source.push_str(&source[..value_span.start]);
    new_source.push_str(&new_value_str);
    new_source.push_str(&source[value_span.end..]);

    shared.source = new_source;
    shared.version += 1;

    // RELEASE WRITE LOCK (implicit)
    Ok(())
}
```

**Safe because:**
- Write lock held for entire operation
- Byte span calculation and source update are atomic
- No interleaving possible

### ✅ Scenario 8: Sequential Updates to Same File

**Example:**
```
T1: update index 0, write to disk
T2: update index 1, write to disk

1. T1: update_macro_by_index(0) - shared = {source: T1_mod, version: 1, disk_source: original}
2. T1: write_to_disk() - writes T1_mod, updates disk_source = T1_mod
3. Shared state: {source: T1_mod, version: 1, disk_source: T1_mod}
4. T2: update_macro_by_index(1) - reads shared.source (T1_mod), does splicing
   - shared = {source: T2_mod, version: 2, disk_source: T1_mod}
5. T2: write_to_disk() - checks disk (T1_mod) == disk_source (T1_mod) ✓
   - writes T2_mod, updates disk_source = T2_mod
```

**Safe because:**
- In-memory `shared.source` is the source of truth
- `disk_source` tracks expected disk state
- T2 applies its changes to T1's modified source
- Both updates are preserved

## Key Design Insights

1. **In-memory state is authoritative**: `shared.source` is the source of truth
2. **Disk is persistence layer**: `write_to_disk` is a separate operation
3. **Write locks serialize modifications**: Only one thread modifies at a time
4. **Character-range splicing composes correctly**: Each update reads latest shared.source
5. **Version-based invalidation**: Thread-local caches stay coherent
6. **Concurrent modification detection**: External changes are caught and rejected

## Potential Issues

### ⚠️ None Identified

After thorough analysis, I found **no race conditions or threading bugs**.

The synchronization strategy is sound and handles all edge cases correctly:
- Concurrent updates to same file: serialized via write lock
- Concurrent reads during writes: blocked until write completes
- Thread-local cache staleness: detected via version checking
- External file modifications: detected and rejected
- Lazy initialization races: prevented by double-check locking

## Testing Recommendations

To gain additional confidence, consider adding tests for:

1. **Concurrent updates from multiple threads**
   ```rust
   #[test]
   fn test_concurrent_updates() {
       // Spawn multiple threads updating different macros
       // Verify all updates are preserved
   }
   ```

2. **Interleaved operations**
   ```rust
   #[test]
   fn test_interleaved_read_write() {
       // Thread 1: continuous reads
       // Thread 2: periodic writes
       // Verify consistency
   }
   ```

3. **Cache invalidation under load**
   ```rust
   #[test]
   fn test_cache_invalidation() {
       // Multiple threads with cached ASTs
       // One thread does updates
       // Verify other threads see updates
   }
   ```

## Conclusion

**The implementation is thread-safe.** The combination of:
- RwLock for shared state
- Thread-local caching with version checking
- Write lock held during entire critical section
- Concurrent modification detection

...provides robust protection against all identified race conditions.
