use std::{cmp::Ordering, collections::HashMap};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use crate::consts::{DUNGEON_ACTIVITY_HASH, DUNGEON_ACTIVITY_MODE, RAID_ACTIVITY_HASH, RAID_ACTIVITY_MODE};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BungieProfile {
    membership_type: usize,
    membership_id: String,
    bungie_global_display_name: String,
    bungie_global_display_name_code: usize,
    cross_save_override: usize,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    pub privacy: usize,
    pub display_name: String,
    pub display_tag: usize,
    pub character_ids: Vec<String>,
}

impl<'de> Deserialize<'de> for ProfileInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Profile {
            profile: _ProfileInfo,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _ProfileInfo {
            data: _ProfileData,
            privacy: usize,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _ProfileData {
            user_info: _UserInfo,
            character_ids: Vec<String>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _UserInfo {
            bungie_global_display_name: String,
            bungie_global_display_name_code: usize,
        }

        let profile = _Profile::deserialize(deserializer)?;
        Ok(Self {
            privacy: profile.profile.privacy,
            display_name: profile.profile.data.user_info.bungie_global_display_name,
            display_tag: profile
                .profile
                .data
                .user_info
                .bungie_global_display_name_code,
            character_ids: profile.profile.data.character_ids,
        })
    }
}

#[derive(Debug)]
pub struct ProfileCurrentActivities {
    pub privacy: usize,
    pub activities: Option<HashMap<String, LatestCharacterActivity>>,
    pub response_minted_timestamp: Option<DateTime<Utc>>,
    pub secondary_components_minted_timestamp: Option<DateTime<Utc>>,
    pub transitory_start_time: Option<DateTime<Utc>>,
}

// Component 1000 is optional and fetched independently. A failed or incomplete
// timing response must never prevent a component 204 status update.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileTransitoryTiming {
    #[serde(default, deserialize_with = "optional_component")]
    pub secondary_components_minted_timestamp: Option<DateTime<Utc>>,
    #[serde(default, rename = "profileTransitoryData", deserialize_with = "transitory_start")]
    pub start_time: Option<DateTime<Utc>>,
}

fn transitory_start<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let component: Option<ProfileTransitoryComponent> = optional_component(deserializer)?;
    Ok(component.and_then(|component| component.data)
        .and_then(|data| data.current_activity)
        .and_then(|activity| activity.start_time))
}

// Optional timing data must not make the existing CharacterActivities parser
// fail when Bungie omits it, restricts it, or returns an unexpected value.
fn optional_component<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(serde_json::from_value(value).ok())
}

#[derive(Deserialize)]
struct ProfileTransitoryComponent {
    data: Option<ProfileTransitoryData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileTransitoryData {
    current_activity: Option<ProfileTransitoryActivity>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileTransitoryActivity {
    #[serde(default, deserialize_with = "optional_component")]
    start_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LatestCharacterActivity {
    pub date_activity_started: DateTime<Utc>,
    pub current_activity_hash: usize,
}

impl PartialOrd for LatestCharacterActivity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.date_activity_started
            .partial_cmp(&other.date_activity_started)
    }
}

impl Ord for LatestCharacterActivity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date_activity_started.cmp(&other.date_activity_started)
    }
}

impl<'de> Deserialize<'de> for ProfileCurrentActivities {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Profile {
            character_activities: _CurrentActivities,
            #[serde(default, deserialize_with = "optional_component")]
            response_minted_timestamp: Option<DateTime<Utc>>,
            #[serde(default, deserialize_with = "optional_component")]
            secondary_components_minted_timestamp: Option<DateTime<Utc>>,
            #[serde(default, deserialize_with = "optional_component")]
            profile_transitory_data: Option<ProfileTransitoryComponent>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _CurrentActivities {
            data: Option<HashMap<String, _CurrentActivity>>,
            privacy: usize,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _CurrentActivity {
            date_activity_started: DateTime<Utc>,
            current_activity_hash: usize,
        }

        let profile = _Profile::deserialize(deserializer)?;
        Ok(Self {
            privacy: profile.character_activities.privacy,
            response_minted_timestamp: profile.response_minted_timestamp,
            secondary_components_minted_timestamp: profile.secondary_components_minted_timestamp,
            transitory_start_time: profile.profile_transitory_data
                .and_then(|component| component.data)
                .and_then(|data| data.current_activity)
                .and_then(|activity| activity.start_time),
            activities: profile.character_activities.data.map(|d| {
                d.into_iter()
                    .map(|e| {
                        (
                            e.0,
                            LatestCharacterActivity {
                                date_activity_started: e.1.date_activity_started,
                                current_activity_hash: e.1.current_activity_hash,
                            },
                        )
                    })
                    .collect()
            }),
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterActivityHistory {
    pub activities: Option<Vec<CompletedActivity>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletedActivity {
    pub period: DateTime<Utc>,
    pub instance_id: String,
    pub activity_hash: usize,
    pub modes: Vec<usize>,
    pub completed: bool,
    pub activity_duration: String,
    pub activity_duration_seconds: usize,
}

impl PartialOrd for CompletedActivity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.period.partial_cmp(&other.period)
    }
}

impl Ord for CompletedActivity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.period.cmp(&other.period)
    }
}

impl<'de> Deserialize<'de> for CompletedActivity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Activity {
            period: DateTime<Utc>,
            activity_details: _ActivityDetails,
            values: _Values,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _ActivityDetails {
            instance_id: String,
            director_activity_hash: usize,
            modes: Vec<usize>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Values {
            completion_reason: _Value,
            completed: _Value,
            activity_duration_seconds: _Value,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Value {
            basic: _BasicValue,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _BasicValue {
            value: f32,
            display_value: String,
        }

        let activity = _Activity::deserialize(deserializer)?;
        Ok(Self {
            period: activity.period,
            instance_id: activity.activity_details.instance_id,
            activity_hash: activity.activity_details.director_activity_hash,
            modes: activity.activity_details.modes,
            completed: activity.values.completed.basic.value == 1.0
                && activity.values.completion_reason.basic.value == 0.0,
            activity_duration: activity
                .values
                .activity_duration_seconds
                .basic
                .display_value,
            activity_duration_seconds: activity.values.activity_duration_seconds.basic.value
                as usize,
        })
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivityInfo {
    pub name: String,
    pub activity_modes: Vec<usize>,
    pub background_image: Option<String>,
}

impl<'de> Deserialize<'de> for ActivityInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Activity {
            original_display_properties: _DisplayProperties,
            activity_mode_types: Option<Vec<usize>>,
            activity_type_hash: usize,
            pgcr_image: Option<String>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _DisplayProperties {
            name: String,
        }

        fn modes_from_hash(hash: usize) -> Vec<usize> {
            match hash {
                RAID_ACTIVITY_HASH => vec![RAID_ACTIVITY_MODE],
                DUNGEON_ACTIVITY_HASH => vec![DUNGEON_ACTIVITY_MODE],
                _ => vec![],
            }
        }

        let activity = _Activity::deserialize(deserializer)?;
        Ok(Self {
            name: activity.original_display_properties.name,
            activity_modes: {
                let mut modes = activity.activity_mode_types.unwrap_or_default();
                for mode in modes_from_hash(activity.activity_type_hash) {
                    if !modes.contains(&mode) {
                        modes.push(mode);
                    }
                }
                modes
            },
            background_image: activity.pgcr_image,
        })
    }
}

#[cfg(test)]
mod timing_tests {
    use super::*;
    use serde_json::json;

    fn profile() -> serde_json::Value {
        json!({
            "characterActivities": { "privacy": 1, "data": {
                "character": {
                    "dateActivityStarted": "2026-09-17T00:00:10Z",
                    "currentActivityHash": 123
                }
            }},
            "responseMintedTimestamp": "2026-09-17T00:01:30Z",
            "secondaryComponentsMintedTimestamp": "2026-09-17T00:01:35Z",
            "profileTransitoryData": { "privacy": 1, "data": {
                "currentActivity": { "startTime": "2026-09-17T00:01:10Z" }
            }}
        })
    }

    #[test]
    fn parses_both_independent_freshness_timestamps_and_start_time() {
        let parsed: ProfileCurrentActivities = serde_json::from_value(profile()).unwrap();
        assert!(parsed.response_minted_timestamp < parsed.secondary_components_minted_timestamp);
        assert_eq!(parsed.transitory_start_time.unwrap().to_rfc3339(), "2026-09-17T00:01:10+00:00");
        assert_eq!(parsed.activities.unwrap()["character"].current_activity_hash, 123);
    }

    #[test]
    fn missing_private_null_and_malformed_transitory_keep_character_data() {
        for optional in [
            serde_json::Value::Null,
            json!({ "privacy": 2 }),
            json!({ "privacy": 2, "data": null }),
            json!({ "data": { "currentActivity": null } }),
            json!({ "data": { "currentActivity": { "startTime": null } } }),
            json!({ "data": { "currentActivity": { "startTime": "not-a-time" } } }),
            json!({ "data": { "currentActivity": { "startTime": 1234 } } }),
            json!({ "data": [] }),
        ] {
            let mut value = profile();
            value["profileTransitoryData"] = optional;
            let parsed: ProfileCurrentActivities = serde_json::from_value(value).unwrap();
            assert!(parsed.transitory_start_time.is_none());
            assert_eq!(parsed.activities.unwrap()["character"].current_activity_hash, 123);
        }
        let mut value = profile();
        value.as_object_mut().unwrap().remove("profileTransitoryData");
        let parsed: ProfileCurrentActivities = serde_json::from_value(value).unwrap();
        assert!(parsed.transitory_start_time.is_none());
    }

    #[test]
    fn absent_or_invalid_freshness_metadata_preserves_legacy_parsing() {
        for optional in [serde_json::Value::Null, json!("invalid"), json!(17)] {
            let mut value = profile();
            value["responseMintedTimestamp"] = optional.clone();
            value["secondaryComponentsMintedTimestamp"] = optional;
            let parsed: ProfileCurrentActivities = serde_json::from_value(value).unwrap();
            assert!(parsed.response_minted_timestamp.is_none());
            assert!(parsed.secondary_components_minted_timestamp.is_none());
            assert!(parsed.activities.is_some());
        }
        let mut value = profile();
        for field in ["responseMintedTimestamp", "secondaryComponentsMintedTimestamp"] {
            value.as_object_mut().unwrap().remove(field);
        }
        let parsed: ProfileCurrentActivities = serde_json::from_value(value).unwrap();
        assert!(parsed.response_minted_timestamp.is_none());
        assert!(parsed.secondary_components_minted_timestamp.is_none());
    }

    #[test]
    fn private_character_data_stays_private_even_with_transitory() {
        let mut value = profile();
        value["characterActivities"] = json!({ "privacy": 2 });
        let parsed: ProfileCurrentActivities = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.privacy, 2);
        assert!(parsed.activities.is_none());
    }

    #[test]
    fn standalone_transitory_does_not_require_character_activities() {
        let mut value = profile();
        value.as_object_mut().unwrap().remove("characterActivities");
        value.as_object_mut().unwrap().remove("responseMintedTimestamp");
        let parsed: ProfileTransitoryTiming = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.start_time.unwrap().to_rfc3339(), "2026-09-17T00:01:10+00:00");
        assert_eq!(parsed.secondary_components_minted_timestamp.unwrap().to_rfc3339(), "2026-09-17T00:01:35+00:00");
    }

    #[test]
    fn standalone_optional_timing_tolerates_missing_and_malformed_data() {
        for value in [json!({}), json!({"profileTransitoryData": null}),
            json!({"profileTransitoryData": {"privacy": 2}}),
            json!({"profileTransitoryData": {"data": {"currentActivity": null}}}),
            json!({"profileTransitoryData": {"data": {"currentActivity": {"startTime": "invalid"}}},
                "secondaryComponentsMintedTimestamp": "invalid"})] {
            let parsed: ProfileTransitoryTiming = serde_json::from_value(value).unwrap();
            assert!(parsed.start_time.is_none());
            assert!(parsed.secondary_components_minted_timestamp.is_none());
        }
    }
}
