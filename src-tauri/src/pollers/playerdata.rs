use std::{sync::Arc, time::Duration};

use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{
    async_runtime::{self, JoinHandle},
    AppHandle, Manager,
};
use tokio::sync::Mutex;

use crate::{
    api::{
        requests::BungieResponseError,
        responses::{ActivityInfo, CompletedActivity, LatestCharacterActivity, ProfileInfo},
        Api, ApiError, Source,
    },
    config::profiles::Profile,
    consts::{DUNGEON_ACTIVITY_MODE, LOSTSECTOR_ACTIVITY_MODE, RAID_ACTIVITY_MODE, STRIKE_ACTIVITY_MODE},
    ConfigContainer,
};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerData {
    current_activity: CurrentActivity,
    activity_history: Vec<CompletedActivity>,
    profile_info: ProfileInfo,
}

#[derive(Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDataStatus {
    last_update: Option<PlayerData>,
    error: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CurrentActivity {
    start_date: DateTime<Utc>,
    activity_hash: usize,
    activity_info: Option<ActivityInfo>,
}

#[derive(Default)]
pub struct PlayerDataPoller {
    task_handle: Option<JoinHandle<()>>,
    current_playerdata: Arc<Mutex<PlayerDataStatus>>,
}

impl PlayerDataPoller {
    pub async fn reset(&mut self, app_handle: AppHandle) {
        if let Some(t) = self.task_handle.as_ref() {
            t.abort();
        }

        {
            let mut lock = self.current_playerdata.lock().await;
            *lock = PlayerDataStatus::default();

            send_data_update(&app_handle, lock.clone());
        }

        let playerdata_clone = self.current_playerdata.clone();

        self.task_handle = Some(async_runtime::spawn(async move {
            let profile = {
                let container = app_handle.state::<ConfigContainer>();
                let lock = container.0.lock().await;

                match &lock.get_profiles().selected_profile {
                    Some(p) => p.clone(),
                    None => {
                        let mut lock = playerdata_clone.lock().await;
                        lock.error = Some("No profile set".to_string());

                        send_data_update(&app_handle, lock.clone());
                        return;
                    }
                }
            };

            let profile_info = loop {
                let result = {
                    let api = app_handle.state::<Api>();
                    let result = api.profile_info_source.lock().await.get(&profile).await;
                    result
                };
                match result {
                    Ok(info) => break info,
                    Err(error) => {
                        {
                            let mut state = playerdata_clone.lock().await;
                            state.error = Some(format!("Failed to get profile info: {error}"));
                            send_data_update(&app_handle, state.clone());
                        }
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            };

            let mut current_activity = CurrentActivity {
                start_date: DateTime::<Utc>::MIN_UTC,
                activity_hash: 0,
                activity_info: None,
            };
            let mut activity_history = Vec::new();

            loop {
                // Initial failures keep retrying; no app restart is required.
                let result = match update_current(&app_handle, &mut current_activity, &profile).await {
                    Ok(_) => update_history(&app_handle, &mut activity_history, &profile).await,
                    Err(error) => Err(error),
                };
                match result {
                    Ok(_) => break,
                    Err(error) => {
                        {
                            let mut state = playerdata_clone.lock().await;
                            state.error = Some(error.to_string());
                            send_data_update(&app_handle, state.clone());
                        }
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
            {
                let mut state = playerdata_clone.lock().await;
                state.error = None;
                state.last_update = Some(PlayerData {
                    current_activity,
                    activity_history,
                    profile_info,
                });
                send_data_update(&app_handle, state.clone());
            }

            // Both futures belong to this task: reset/Exit cancels both.
            // A slow history request must not stop current-activity polling.
            let current_updates = async {
                loop {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    let mut current = playerdata_clone.lock().await
                        .last_update.as_ref().unwrap().current_activity.clone();
                    let result = update_current(&app_handle, &mut current, &profile).await;
                    let mut state = playerdata_clone.lock().await;
                    match result {
                        Ok(changed) => {
                            let recovered = state.error.take().is_some();
                            if changed {
                                state.last_update.as_mut().unwrap().current_activity = current;
                            }
                            if changed || recovered {
                                send_data_update(&app_handle, state.clone());
                            }
                        }
                        Err(error) => {
                            state.error = Some(error.to_string());
                            send_data_update(&app_handle, state.clone());
                        }
                    }
                }
            };

            let history_updates = async {
                loop {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    let mut history = playerdata_clone.lock().await
                        .last_update.as_ref().unwrap().activity_history.clone();
                    let result = update_history(&app_handle, &mut history, &profile).await;
                    let mut state = playerdata_clone.lock().await;
                    match result {
                        Ok(changed) => {
                            if changed {
                                // Update only history; preserve newer current-activity data.
                                state.last_update.as_mut().unwrap().activity_history = history;
                                send_data_update(&app_handle, state.clone());
                            }
                        }
                        Err(error) => {
                            state.error = Some(error.to_string());
                            send_data_update(&app_handle, state.clone());
                        }
                    }
                }
            };

            tokio::join!(current_updates, history_updates);
        }));
    }

    pub fn get_data(&mut self) -> Option<PlayerDataStatus> {
        match &self.current_playerdata.try_lock() {
            Ok(p) => Some((*p).clone()),
            Err(_) => None,
        }
    }
}

fn send_data_update(handle: &AppHandle, data: PlayerDataStatus) {
    if let Some(o) = handle.get_window("overlay") {
        let _ = o.emit("playerdata_update", data.clone());
    }

    if let Some(o) = handle.get_window("details") {
        let _ = o.emit("playerdata_update", data);
    }
}

async fn update_current(
    handle: &AppHandle,
    last_activity: &mut CurrentActivity,
    profile: &Profile,
) -> Result<bool> {
    let current_activities = Api::get_profile_activities(profile).await?;

    let activities = match current_activities.activities {
        Some(a) => a,
        None => bail!("Profile is private"),
    };

    let (characters, activities): (Vec<String>, Vec<LatestCharacterActivity>) =
        activities.into_iter().unzip();

    let latest_activity = activities
        .into_iter()
        .max()
        .ok_or(anyhow!("No character data for profile"))?;

    if !should_refresh_current(last_activity, &latest_activity) {
        return Ok(false);
    }

    let api = handle.state::<Api>();
    api.profile_info_source.lock().await.set_characters(profile, characters);

    let activity_info = if latest_activity.current_activity_hash == 0 {
        None
    } else {
        match api.activity_info_source.lock().await
            .get(&latest_activity.current_activity_hash).await {
            Ok(info) if !info.name.is_empty() => Some(info),
            Ok(_) | Err(ApiError::ResponseError(BungieResponseError::ResponseMissing)) => None,
            Err(error) => return Err(error.into()),
        }
    };

    // Commit the complete state together, including hash=0 when entering orbit.
    *last_activity = CurrentActivity {
        start_date: latest_activity.date_activity_started,
        activity_hash: latest_activity.current_activity_hash,
        activity_info,
    };
    Ok(true)
}

fn should_refresh_current(last: &CurrentActivity, latest: &LatestCharacterActivity) -> bool {
    if latest.date_activity_started < last.start_date {
        return false;
    }
    latest.date_activity_started > last.start_date
        || latest.current_activity_hash != last.activity_hash
        || (latest.current_activity_hash != 0 && last.activity_info.is_none())
}

async fn update_history(
    handle: &AppHandle,
    last_history: &mut Vec<CompletedActivity>,
    profile: &Profile,
) -> Result<bool> {
    let api = handle.state::<Api>();

    let profile_info = api.profile_info_source.lock().await.get(profile).await?;
    let cached_modes = api
        .activity_info_source
        .lock()
        .await
        .cached_modes_snapshot();

    let mut past_activities: Vec<CompletedActivity> = Vec::new();

    let cutoff = {
        let now = Utc::now();
        let naive_cutoff = now
            .date_naive()
            .and_hms_opt(17, 0, 0)
            .ok_or(anyhow!("There is no 5PM UTC today?"))?;

        let mut time = DateTime::<Utc>::from_utc(naive_cutoff, Utc);

        if time > now {
            time -= chrono::Duration::days(1);
        }

        time
    };

    for character_id in profile_info.character_ids.iter() {
        let mut page = 0;

        loop {
            let history = Api::get_activity_history(profile, character_id, page).await?;

            let activities = match history.activities {
                Some(a) if !a.is_empty() => a,
                _ => break,
            };

            let mut includes_past_cutoff = false;

            for mut activity in activities.into_iter() {
                if activity.period < cutoff {
                    includes_past_cutoff = true;
                    continue;
                }

                supplement_cached_modes(
                    &mut activity.modes,
                    cached_modes.get(&activity.activity_hash).map(Vec::as_slice),
                );

                if has_tracked_mode(&activity.modes) {
                    past_activities.push(activity);
                }
            }

            if includes_past_cutoff {
                break;
            }

            page += 1;
        }
    }

    Ok(replace_history_if_changed(last_history, past_activities))
}

fn supplement_cached_modes(modes: &mut Vec<usize>, cached: Option<&[usize]>) {
    if !has_tracked_mode(modes) {
        if let Some(cached) = cached {
            for mode in cached {
                if !modes.contains(mode) {
                    modes.push(*mode);
                }
            }
        }
    }
}

fn has_tracked_mode(modes: &[usize]) -> bool {
    modes.iter().any(|m| {
        *m == RAID_ACTIVITY_MODE
            || *m == DUNGEON_ACTIVITY_MODE
            || *m == STRIKE_ACTIVITY_MODE
            || *m == LOSTSECTOR_ACTIVITY_MODE
    })
}

fn replace_history_if_changed(
    last_history: &mut Vec<CompletedActivity>,
    mut activities: Vec<CompletedActivity>,
) -> bool {
    // Include a stable tie-breaker for runs with the same start time.
    activities.sort_by(|a, b| {
        b.period.cmp(&a.period).then_with(|| a.instance_id.cmp(&b.instance_id))
    });

    // Compare complete records: older arrivals and corrected results matter too.
    if *last_history == activities {
        return false;
    }

    *last_history = activities;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn activity(seconds: i64, id: &str) -> CompletedActivity {
        CompletedActivity {
            period: DateTime::parse_from_rfc3339("2026-09-10T00:00:00Z")
                .unwrap().with_timezone(&Utc) + chrono::Duration::seconds(seconds),
            instance_id: id.to_string(),
            activity_hash: 123,
            modes: vec![DUNGEON_ACTIVITY_MODE],
            completed: false,
            activity_duration: "1:00".to_string(),
            activity_duration_seconds: 60,
        }
    }

    #[test]
    fn accepts_late_older_activity() {
        let newest = activity(20, "newest");
        let mut history = vec![newest.clone()];
        assert!(replace_history_if_changed(
            &mut history, vec![activity(10, "late"), newest]
        ));
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].instance_id, "newest");
    }

    #[test]
    fn accepts_corrected_completion_and_duration() {
        let mut corrected = activity(20, "same");
        let mut history = vec![corrected.clone()];
        corrected.completed = true;
        corrected.activity_duration_seconds = 61;
        corrected.activity_duration = "1:01".to_string();
        assert!(replace_history_if_changed(&mut history, vec![corrected.clone()]));
        assert_eq!(history, vec![corrected]);
    }

    #[test]
    fn removes_old_records_and_clears_at_reset() {
        let newest = activity(20, "newest");
        let mut history = vec![newest.clone(), activity(10, "old")];
        assert!(replace_history_if_changed(&mut history, vec![newest]));
        assert!(replace_history_if_changed(&mut history, vec![]));
        assert!(history.is_empty());
        assert!(!replace_history_if_changed(&mut history, vec![]));
    }

    #[test]
    fn unchanged_history_ignores_response_order_even_with_equal_times() {
        let a = activity(20, "a");
        let b = activity(20, "b");
        let mut history = vec![];
        assert!(replace_history_if_changed(&mut history, vec![b.clone(), a.clone()]));
        assert!(!replace_history_if_changed(&mut history, vec![a, b]));
    }

    #[test]
    fn cache_supplements_empty_and_generic_modes_without_duplicates() {
        for mut modes in [vec![], vec![7]] {
            supplement_cached_modes(&mut modes, Some(&[DUNGEON_ACTIVITY_MODE]));
            assert!(has_tracked_mode(&modes));
            let once = modes.clone();
            supplement_cached_modes(&mut modes, Some(&[DUNGEON_ACTIVITY_MODE]));
            assert_eq!(modes, once);
        }
        let mut generic = vec![7];
        supplement_cached_modes(&mut generic, None);
        assert_eq!(generic, vec![7]);
        let mut known = vec![RAID_ACTIVITY_MODE];
        supplement_cached_modes(&mut known, Some(&[DUNGEON_ACTIVITY_MODE]));
        assert_eq!(known, vec![RAID_ACTIVITY_MODE]);
    }

    #[test]
    fn incomplete_modes_require_fallback_but_known_modes_do_not() {
        assert!(!has_tracked_mode(&[]));
        assert!(!has_tracked_mode(&[7]));
        for mode in [RAID_ACTIVITY_MODE, DUNGEON_ACTIVITY_MODE,
                     STRIKE_ACTIVITY_MODE, LOSTSECTOR_ACTIVITY_MODE] {
            assert!(has_tracked_mode(&[7, mode]));
        }
    }

    #[test]
    fn valid_hash_recovers_after_missing_info_at_same_start_time() {
        let last = CurrentActivity {
            start_date: activity(20, "time").period,
            activity_hash: 0,
            activity_info: None,
        };
        let latest = LatestCharacterActivity {
            date_activity_started: last.start_date,
            current_activity_hash: 456,
        };
        assert!(should_refresh_current(&last, &latest));
    }

    #[test]
    fn missing_definition_can_recover_without_a_new_activity_timestamp() {
        let last = CurrentActivity {
            start_date: activity(20, "time").period,
            activity_hash: 456,
            activity_info: None,
        };
        let latest = LatestCharacterActivity {
            date_activity_started: last.start_date,
            current_activity_hash: 456,
        };
        assert!(should_refresh_current(&last, &latest));
    }

    #[test]
    fn current_guard_ignores_older_responses_and_unchanged_orbit() {
        let last = CurrentActivity {
            start_date: activity(20, "time").period,
            activity_hash: 0,
            activity_info: None,
        };
        let mut latest = LatestCharacterActivity {
            date_activity_started: last.start_date,
            current_activity_hash: 0,
        };
        assert!(!should_refresh_current(&last, &latest));
        latest.current_activity_hash = 456;
        latest.date_activity_started = activity(10, "old").period;
        assert!(!should_refresh_current(&last, &latest));
    }

    #[test]
    fn same_dungeon_reentry_updates_timer_but_unchanged_run_does_not() {
        let last = CurrentActivity {
            start_date: activity(20, "time").period,
            activity_hash: 456,
            activity_info: Some(ActivityInfo {
                name: "Dungeon".to_string(),
                activity_modes: vec![DUNGEON_ACTIVITY_MODE],
                background_image: None,
            }),
        };
        let mut latest = LatestCharacterActivity {
            date_activity_started: last.start_date,
            current_activity_hash: 456,
        };
        assert!(!should_refresh_current(&last, &latest));
        latest.date_activity_started = activity(30, "next").period;
        assert!(should_refresh_current(&last, &latest));
    }
}
