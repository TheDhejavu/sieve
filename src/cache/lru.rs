pub use lru::LruCache;
pub use std::sync::Arc;
pub use tokio::sync::RwLock;

#[macro_export]
macro_rules! lru_cache {
    ($size:expr) => {
        Arc::new(RwLock::new(LruCache::new(
            std::num::NonZeroUsize::new($size).unwrap(),
        )))
    };
}
