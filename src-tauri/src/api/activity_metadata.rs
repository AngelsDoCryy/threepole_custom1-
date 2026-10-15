use std::{collections::HashMap, future::Future, sync::{Arc, Mutex}, time::{Duration, Instant}};

use super::{ActivityModeCache, ApiError, requests::{make_request, BungieRequest}, responses::ActivityInfo};

const MAX_ENTRIES: usize = 256;
const RETRY_DELAY: Duration = Duration::from_secs(30);

struct Entry {
    info: Option<ActivityInfo>,
    retry_at: Instant,
    used_at: Instant,
    complete: bool,
    attempts: u32,
}

#[derive(Default)]
struct Cache {
    entries: HashMap<usize, Entry>,
    modes: ActivityModeCache,
}

/// Cache locks cover memory only. One async gate serializes manifest requests,
/// including details IPC; it never gates current status or cached mode reads.
pub struct ActivityInfoSource {
    cache: Arc<Mutex<Cache>>,
    request_gate: tokio::sync::Mutex<()>,
    persist_gate: Arc<Mutex<()>>,
    persist: bool,
}

impl Default for ActivityInfoSource {
    fn default() -> Self {
        Self::new(ActivityModeCache::load(), true)
    }
}

impl ActivityInfoSource {
    #[cfg(test)]
    pub(crate) fn empty() -> Self { Self::new(ActivityModeCache::default(), false) }

    fn new(modes: ActivityModeCache, persist: bool) -> Self {
        Self {
            cache: Arc::new(Mutex::new(Cache { modes, ..Cache::default() })),
            request_gate: tokio::sync::Mutex::new(()),
            persist_gate: Arc::new(Mutex::new(())),
            persist,
        }
    }

    pub fn peek(&self, hash: usize) -> Option<ActivityInfo> {
        let mut cache = self.cache.lock().unwrap();
        cache.entries.get_mut(&hash).and_then(|entry| {
            entry.used_at = Instant::now();
            entry.info.clone()
        })
    }

    pub fn cached_modes_snapshot(&self) -> HashMap<usize, Vec<usize>> {
        self.cache.lock().unwrap().modes.modes.clone()
    }

    pub fn cached_modes(&self, hash: usize) -> Vec<usize> {
        self.cache.lock().unwrap().modes.modes.get(&hash).cloned().unwrap_or_default()
    }

    fn ready(&self, hash: usize) -> Option<Result<ActivityInfo, ApiError>> {
        let mut cache = self.cache.lock().unwrap();
        let entry = cache.entries.get_mut(&hash)?;
        entry.used_at = Instant::now();
        if entry.complete || Instant::now() < entry.retry_at {
            Some(entry.info.clone().filter(|info| !info.name.trim().is_empty())
                .ok_or(ApiError::MetadataUnavailable))
        } else {
            None
        }
    }

    pub async fn get(&self, hash: &usize) -> Result<ActivityInfo, ApiError> {
        self.get_with(*hash, || async {
            let value = make_request(BungieRequest::GetDestinyActivityDefinition { activity_hash: *hash })
                .await.map_err(ApiError::ResponseError)?;
            serde_json::from_value(value).map_err(ApiError::ResponseDeserializeError)
        }).await
    }

    pub(crate) async fn get_with<F, Fut>(&self, hash: usize, fetch: F) -> Result<ActivityInfo, ApiError>
    where F: FnOnce() -> Fut, Fut: Future<Output = Result<ActivityInfo, ApiError>> {
        if hash == 0 { return Err(ApiError::MetadataUnavailable); }
        let cached = {
            let mut cache = self.cache.lock().unwrap();
            cache.entries.get_mut(&hash).filter(|entry| entry.complete).and_then(|entry| {
                entry.used_at = Instant::now();
                entry.info.clone()
            })
        };
        if let Some(info) = cached { return Ok(info); }
        let _request = self.request_gate.lock().await;
        // Coalesce concurrent misses and failed/empty results as well as successes.
        if let Some(result) = self.ready(hash) { return result; }
        let retry_delay = {
            let mut cache = self.cache.lock().unwrap();
            if !cache.entries.contains_key(&hash) && cache.entries.len() >= MAX_ENTRIES {
                if let Some(oldest) = cache.entries.iter().min_by_key(|(_, e)| e.used_at).map(|(k, _)| *k) {
                    cache.entries.remove(&oldest);
                }
            }
            let now = Instant::now();
            let entry = cache.entries.entry(hash).or_insert(Entry {
                info: None, retry_at: now, used_at: now, complete: false, attempts: 0,
            });
            // Cancellation also leaves a cooldown; no wedged in-flight flag.
            let delay = Duration::from_secs((RETRY_DELAY.as_secs() * (1u64 << entry.attempts.min(4))).min(300));
            entry.attempts = entry.attempts.saturating_add(1);
            entry.retry_at = now + delay;
            delay
        };
        let result = fetch().await.map(|mut info| {
            info.name = info.name.trim().to_owned();
            info
        });
        let persist = {
            let mut cache = self.cache.lock().unwrap();
            let mut changed = false;
            if let Ok(info) = &result {
                changed = cache.modes.record_raid_or_dungeon(hash, &info.activity_modes);
            }
            let entry = cache.entries.get_mut(&hash).unwrap();
            entry.retry_at = Instant::now() + retry_delay;
            if let Ok(info) = &result {
                entry.complete = !info.name.trim().is_empty()
                    && info.activity_modes.iter().any(|mode| *mode != 0 && *mode != 7);
                entry.info = Some(info.clone());
            }
            changed && self.persist
        };
        if persist {
            let cache = self.cache.clone();
            let gate = self.persist_gate.clone();
            // Existing runtime blocking pool, no disk I/O under the cache lock.
            // Snapshot after acquiring the writer gate prevents stale disk writes
            // if the caller is cancelled while a previous write completes.
            let _ = tauri::async_runtime::spawn_blocking(move || {
                let _writer = gate.lock().unwrap();
                let modes = cache.lock().unwrap().modes.clone();
                modes.persist();
            }).await;
        }
        result.and_then(|info| if info.name.trim().is_empty() {
            Err(ApiError::MetadataUnavailable)
        } else { Ok(info) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::requests::BungieResponseError;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn info(name: &str, modes: Vec<usize>) -> ActivityInfo {
        ActivityInfo { name: name.into(), activity_modes: modes, background_image: None }
    }

    fn expire(source: &ActivityInfoSource, hash: usize) {
        source.cache.lock().unwrap().entries.get_mut(&hash).unwrap().retry_at = Instant::now();
    }

    #[tokio::test]
    async fn concurrent_misses_share_one_request_and_cache_reads_do_not_wait() {
        let source = ActivityInfoSource::empty();
        let calls = AtomicUsize::new(0);
        let (start_tx, start_rx) = tokio::sync::oneshot::channel();
        let (end_tx, end_rx) = tokio::sync::oneshot::channel();
        let first = source.get_with(123, || async {
            calls.fetch_add(1, Ordering::SeqCst);
            start_tx.send(()).unwrap();
            end_rx.await.unwrap();
            Ok(info("Dungeon", vec![82]))
        });
        let second = async {
            start_rx.await.unwrap();
            assert!(source.peek(123).is_none());
            assert!(source.cached_modes_snapshot().is_empty());
            end_tx.send(()).unwrap();
            source.get_with(123, || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(info("duplicate", vec![82]))
            }).await
        };
        let (a, b) = tokio::time::timeout(Duration::from_secs(2), async {
            tokio::join!(first, second)
        }).await.unwrap();
        assert_eq!(a.unwrap(), b.unwrap());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(source.cached_modes(123), vec![82]);
    }

    #[tokio::test]
    async fn failed_empty_and_incomplete_entries_recover_with_bounded_retries() {
        let invalid = serde_json::from_str::<ActivityInfo>("{}").unwrap_err();
        let results = vec![
            Err(ApiError::ResponseDeserializeError(invalid)),
            Err(ApiError::ResponseError(BungieResponseError::ResponseMissing)),
            Err(ApiError::ResponseError(BungieResponseError::NetworkError(anyhow::anyhow!("offline")))),
            Ok(info("", vec![82])), Ok(info("   ", vec![82])),
            Ok(info("Dungeon", vec![])), Ok(info("Dungeon", vec![7])),
        ];
        for result in results {
            let source = ActivityInfoSource::empty();
            let _ = source.get_with(123, || async { result }).await;
            let _ = source.get_with(123, || async { panic!("cooldown must prevent a second fetch") }).await;
            assert!(!source.cache.lock().unwrap().entries[&123].complete);
            expire(&source, 123);
            source.get_with(123, || async { Ok(info("Recovered", vec![82])) }).await.unwrap();
            assert_eq!(source.peek(123).unwrap().name, "Recovered");
            assert!(source.cache.lock().unwrap().entries[&123].complete);
        }
    }

    #[tokio::test]
    async fn repeated_failures_back_off_to_five_minutes_and_cache_is_bounded() {
        let source = ActivityInfoSource::empty();
        for delay in [30, 60, 120, 240, 300, 300] {
            let before = Instant::now();
            let _ = source.get_with(123, || async { Err(ApiError::MetadataUnavailable) }).await;
            assert!(source.cache.lock().unwrap().entries[&123].retry_at >= before + Duration::from_secs(delay));
            expire(&source, 123);
        }
        for hash in 1..=MAX_ENTRIES + 2 {
            let _ = source.get_with(hash, || async { Err(ApiError::MetadataUnavailable) }).await;
        }
        assert_eq!(source.cache.lock().unwrap().entries.len(), MAX_ENTRIES);
    }

    #[tokio::test]
    async fn cancelled_fetch_releases_gate_and_can_recover_after_cooldown() {
        let source = ActivityInfoSource::empty();
        let result = tokio::time::timeout(Duration::from_millis(5), source.get_with(123, || async {
            std::future::pending::<Result<ActivityInfo, ApiError>>().await
        })).await;
        assert!(result.is_err());
        source.get_with(456, || async { Ok(info("Other", vec![4])) }).await.unwrap();
        let _ = source.get_with(123, || async { panic!("cancelled request retains cooldown") }).await;
        expire(&source, 123);
        source.get_with(123, || async { Ok(info("Recovered", vec![82])) }).await.unwrap();
    }

    #[tokio::test]
    async fn orbit_never_requests_a_definition() {
        let source = ActivityInfoSource::empty();
        assert!(source.get_with(0, || async { panic!("orbit has no manifest lookup") }).await.is_err());
        assert!(source.cache.lock().unwrap().entries.is_empty());
    }
}
