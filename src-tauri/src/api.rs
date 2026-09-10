use std::error::Error;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    fs::{create_dir_all, read_to_string},
    hash::Hash,
    path::PathBuf,
};

use async_trait::async_trait;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use self::{
    requests::{make_request, BungieRequest, BungieResponseError},
    responses::{
        ActivityInfo, BungieProfile, CharacterActivityHistory, ProfileCurrentActivities,
        ProfileInfo,
    },
};
use crate::config::profiles::Profile;
use crate::consts::{CONFIG_DIR_NAME, DUNGEON_ACTIVITY_MODE, RAID_ACTIVITY_MODE};

pub mod requests;
pub mod responses;

#[derive(Debug)]
pub enum ApiError {
    ResponseDeserializeError(serde_json::Error),
    ResponseError(BungieResponseError),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::ResponseDeserializeError(e) => {
                write!(f, "Failed to parse response object: {}", e)
            }
            ApiError::ResponseError(e) => e.fmt(f),
        }
    }
}

impl Error for ApiError {}

#[async_trait]
pub trait Source<K: Hash + Eq + Clone + Send + Sync, V: Clone + Send> {
    async fn get(&mut self, key: &K) -> Result<V, ApiError>
    where
        K: 'async_trait,
    {
        let cache = self.cache();

        if cache.contains_key(&key) {
            return Ok(cache.get(&key).unwrap().clone());
        }

        let value_fut = Self::get_value(key.clone());

        let value = value_fut.await?;

        cache.insert(key.clone(), value.clone());

        Ok(value)
    }

    async fn get_value(key: K) -> Result<V, ApiError>;

    fn cache(&mut self) -> &mut HashMap<K, V>;
}

#[derive(Default)]
pub struct ProfileInfoSource {
    cache: HashMap<Profile, ProfileInfo>,
}

impl ProfileInfoSource {
    pub fn set_characters(&mut self, profile: &Profile, characters: Vec<String>) {
        if let Some(p) = self.cache.get_mut(profile) {
            p.character_ids = characters;
        }
    }
}

#[async_trait]
impl Source<Profile, ProfileInfo> for ProfileInfoSource {
    async fn get_value(profile: Profile) -> Result<ProfileInfo, ApiError> {
        let res_val = make_request(BungieRequest::GetProfile {
            membership_type: profile.account_platform,
            membership_id: &profile.account_id,
            component: 100,
        })
        .await
        .map_err(|e| ApiError::ResponseError(e))?;

        serde_json::from_value(res_val).map_err(|e| ApiError::ResponseDeserializeError(e))
    }

    fn cache(&mut self) -> &mut HashMap<Profile, ProfileInfo> {
        &mut self.cache
    }
}

#[derive(Serialize, Deserialize, Default)]
struct ActivityModeCache {
    modes: HashMap<usize, Vec<usize>>,
}

impl ActivityModeCache {
    fn path() -> Option<PathBuf> {
        BaseDirs::new().map(|d| {
            let mut path = d.data_dir().to_owned();
            path.push(CONFIG_DIR_NAME);
            path.push("activity-mode-cache.json");
            path
        })
    }

    fn load() -> Self {
        let Some(path) = Self::path() else {
            return Self::default();
        };

        read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn persist(&self) {
        let Some(path) = Self::path() else {
            return;
        };

        if let Some(parent) = path.parent() {
            if create_dir_all(parent).is_err() {
                return;
            }
        }

        if let Ok(serialized) = serde_json::to_string(self) {
            let _ = std::fs::write(path, serialized);
        }
    }

    fn record_raid_or_dungeon(&mut self, activity_hash: usize, modes: &[usize]) {
        let tracked_modes: Vec<usize> = modes
            .iter()
            .copied()
            .filter(|m| *m == RAID_ACTIVITY_MODE || *m == DUNGEON_ACTIVITY_MODE)
            .collect();

        if tracked_modes.is_empty() || self.modes.get(&activity_hash) == Some(&tracked_modes) {
            return;
        }

        self.modes.insert(activity_hash, tracked_modes);
        self.persist();
    }
}

pub struct ActivityInfoSource {
    cache: HashMap<usize, ActivityInfo>,
    mode_cache: ActivityModeCache,
}

impl Default for ActivityInfoSource {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
            mode_cache: ActivityModeCache::load(),
        }
    }
}

impl ActivityInfoSource {
    pub fn cached_modes_snapshot(&self) -> HashMap<usize, Vec<usize>> {
        self.mode_cache.modes.clone()
    }
}

#[async_trait]
impl Source<usize, ActivityInfo> for ActivityInfoSource {
    async fn get(&mut self, key: &usize) -> Result<ActivityInfo, ApiError> {
        if let Some(value) = self.cache.get(key) {
            return Ok(value.clone());
        }

        let value = Self::get_value(*key).await?;

        self.mode_cache
            .record_raid_or_dungeon(*key, &value.activity_modes);
        self.cache.insert(*key, value.clone());

        Ok(value)
    }

    async fn get_value(activity_hash: usize) -> Result<ActivityInfo, ApiError> {
        let res_val = make_request(BungieRequest::GetDestinyActivityDefinition { activity_hash })
            .await
            .map_err(|e| ApiError::ResponseError(e))?;

        serde_json::from_value(res_val).map_err(|e| ApiError::ResponseDeserializeError(e))
    }

    fn cache(&mut self) -> &mut HashMap<usize, ActivityInfo> {
        &mut self.cache
    }
}

#[derive(Default)]
pub struct Api {
    pub profile_info_source: Mutex<ProfileInfoSource>,
    pub activity_info_source: Mutex<ActivityInfoSource>,
}

impl Api {
    pub async fn search_profile(
        display_name: &String,
        display_name_code: usize,
    ) -> Result<Vec<BungieProfile>, ApiError> {
        let res_val = make_request(BungieRequest::SearchDestinyPlayerByBungieName {
            display_name: display_name,
            display_name_code,
        })
        .await
        .map_err(|e| ApiError::ResponseError(e))?;

        serde_json::from_value(res_val).map_err(|e| ApiError::ResponseDeserializeError(e))
    }

    pub async fn get_profile_activities(
        profile: &Profile,
    ) -> Result<ProfileCurrentActivities, ApiError> {
        let res_val = make_request(BungieRequest::GetProfile {
            membership_type: profile.account_platform,
            membership_id: &profile.account_id,
            component: 204,
        })
        .await
        .map_err(|e| ApiError::ResponseError(e))?;

        serde_json::from_value(res_val).map_err(|e| ApiError::ResponseDeserializeError(e))
    }

    pub async fn get_activity_history(
        profile: &Profile,
        character_id: &String,
        page: usize,
    ) -> Result<CharacterActivityHistory, ApiError> {
        let res_val = make_request(BungieRequest::GetActivityHistory {
            membership_type: profile.account_platform,
            membership_id: &profile.account_id,
            character_id: character_id,
            page,
        })
        .await
        .map_err(|e| ApiError::ResponseError(e))?;

        serde_json::from_value(res_val).map_err(|e| ApiError::ResponseDeserializeError(e))
    }
}
