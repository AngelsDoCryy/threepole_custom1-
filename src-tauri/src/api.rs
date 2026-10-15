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
        ProfileInfo, ProfileTransitoryTiming,
    },
};
use crate::config::profiles::Profile;
use crate::consts::{CONFIG_DIR_NAME, DUNGEON_ACTIVITY_MODE, RAID_ACTIVITY_MODE};

pub mod requests;
pub mod responses;
mod activity_metadata;
pub use activity_metadata::ActivityInfoSource;

#[derive(Debug)]
pub enum ApiError {
    ResponseDeserializeError(serde_json::Error),
    ResponseError(BungieResponseError),
    MetadataUnavailable,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::ResponseDeserializeError(e) => {
                write!(f, "Failed to parse response object: {}", e)
            }
            ApiError::ResponseError(e) => e.fmt(f),
            ApiError::MetadataUnavailable => write!(f, "Activity metadata is temporarily unavailable"),
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
            components: &[100],
        })
        .await
        .map_err(|e| ApiError::ResponseError(e))?;

        serde_json::from_value(res_val).map_err(|e| ApiError::ResponseDeserializeError(e))
    }

    fn cache(&mut self) -> &mut HashMap<Profile, ProfileInfo> {
        &mut self.cache
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
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
        let path = match Self::path() {
            Some(path) => path,
            None => return Self::default(),
        };

        read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn persist(&self) {
        let path = match Self::path() {
            Some(path) => path,
            None => return,
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

    fn record_raid_or_dungeon(&mut self, activity_hash: usize, modes: &[usize]) -> bool {
        let tracked_modes: Vec<usize> = modes
            .iter()
            .copied()
            .filter(|m| *m == RAID_ACTIVITY_MODE || *m == DUNGEON_ACTIVITY_MODE)
            .collect();

        if tracked_modes.is_empty() || self.modes.get(&activity_hash) == Some(&tracked_modes) {
            return false;
        }

        self.modes.insert(activity_hash, tracked_modes);
        true
    }
}

#[derive(Default)]
pub struct Api {
    pub profile_info_source: Mutex<ProfileInfoSource>,
    pub activity_info_source: ActivityInfoSource,
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
            // Keep the original status request independent of optional timing.
            // Orbit/activity identity must not wait for component 1000.
            components: &[204],
        })
        .await
        .map_err(|e| ApiError::ResponseError(e))?;

        serde_json::from_value(res_val).map_err(|e| ApiError::ResponseDeserializeError(e))
    }

    pub async fn get_profile_transitory(
        profile: &Profile,
    ) -> Result<ProfileTransitoryTiming, ApiError> {
        let res_val = make_request(BungieRequest::GetProfile {
            membership_type: profile.account_platform,
            membership_id: &profile.account_id,
            components: &[1000],
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
