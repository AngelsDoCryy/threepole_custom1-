use chrono::{DateTime, Utc};

use crate::api::responses::LatestCharacterActivity;

// A few timestamps per selected profile, owned by the existing poller task.
// CharacterActivities and Transitory have independent Bungie freshness clocks.
// Never compare a raw character timestamp with a timer already advanced by
// Transitory: doing that can prevent a later activity hash/orbit from arriving.
#[derive(Default)]
pub(super) struct ActivityTiming {
    character: Option<LatestCharacterActivity>,
    primary_minted: Option<DateTime<Utc>>,
    primary_watermark: Option<DateTime<Utc>>,
    secondary_minted: Option<DateTime<Utc>>,
    transitory_start: Option<DateTime<Utc>>,
    selected_start: Option<DateTime<Utc>>,
}

impl ActivityTiming {
    pub(super) fn resolve(
        &mut self,
        latest: LatestCharacterActivity,
        primary_minted: Option<DateTime<Utc>>,
        secondary_minted: Option<DateTime<Utc>>,
        transitory_start: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Option<LatestCharacterActivity> {
        let primary_minted = primary_minted.filter(|time| valid_time(*time, now));
        let secondary_minted = secondary_minted.filter(|time| valid_time(*time, now));

        let older_primary = match (primary_minted, self.primary_watermark) {
            (Some(incoming), Some(accepted)) => incoming < accepted,
            _ => false,
        };
        let older_character = self.character.as_ref().map_or(false, |accepted| {
            latest.date_activity_started < accepted.date_activity_started
        });
        let newer_primary = match (primary_minted, self.primary_watermark) {
            (Some(incoming), Some(accepted)) => incoming > accepted,
            (Some(_), None) => true,
            _ => false,
        };

        let valid_character = valid_time(latest.date_activity_started, now)
            || (latest.current_activity_hash == 0 && latest.date_activity_started.timestamp() == 0);
        // dateActivityStarted is an activity start, not a status-generation
        // timestamp. A newer status can report orbit with an older/epoch start,
        // or another activity whose start predates the previous observation.
        // Use response freshness first; retain the conservative legacy fallback
        // only when freshness cannot establish that this is a newer snapshot.
        if !older_primary && (!older_character || newer_primary) && valid_character {
            let same_hash = self.character.as_ref().map_or(false, |accepted| {
                latest.current_activity_hash == accepted.current_activity_hash
            });
            let unchanged_character = self.character.as_ref() == Some(&latest);
            self.selected_start = Some(if same_hash && latest.current_activity_hash != 0 {
                self.selected_start.unwrap_or(latest.date_activity_started)
                    .max(latest.date_activity_started)
            } else {
                latest.date_activity_started
            });
            // Missing freshness metadata still permits the original 204 path,
            // but cannot certify a changed identity for Transitory timing.
            if primary_minted.is_some() || !unchanged_character {
                self.primary_minted = primary_minted;
            }
            // A legacy response may change identity without a mint timestamp.
            // Keep the last known freshness boundary even in that case.
            if primary_minted.is_some() {
                self.primary_watermark = primary_minted;
            }
            self.character = Some(latest);
        }

        if let Some(minted) = secondary_minted {
            if self.secondary_minted.map_or(true, |accepted| minted > accepted) {
                self.secondary_minted = Some(minted);
                self.transitory_start = transitory_start
                    .filter(|start| valid_time(*start, now) && *start <= minted);
            }
        }

        let character = self.character.as_ref()?;
        if character.current_activity_hash == 0 {
            // Consume the optional freshness watermark, but do not carry a
            // previous run's timing through an observed return to orbit.
            self.transitory_start = None;
        } else {
            if let (Some(start), Some(primary)) = (self.transitory_start, self.primary_minted) {
                // Transitory does not include an activity hash. Require a 204
                // snapshot generated at/after its start to avoid pairing a
                // known older identity snapshot with a newer run's timer.
                // Older Transitory times may be from the previous run; keep
                // the original 204 timing in that case (including late joins).
                if start >= character.date_activity_started && start <= primary {
                    self.selected_start = Some(self.selected_start.unwrap_or(start).max(start));
                }
            }
        }

        Some(LatestCharacterActivity {
            date_activity_started: self.selected_start.unwrap_or(character.date_activity_started),
            current_activity_hash: character.current_activity_hash,
        })
    }
}

fn valid_time(time: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    time.timestamp() > 0 && time <= now
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(seconds: i64) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-09-17T00:00:00Z").unwrap().with_timezone(&Utc)
            + chrono::Duration::seconds(seconds)
    }

    fn sample(
        tracker: &mut ActivityTiming, hash: usize, started: i64,
        primary: Option<i64>, secondary: Option<i64>, transitory: Option<i64>,
    ) -> LatestCharacterActivity {
        tracker.resolve(
            LatestCharacterActivity { current_activity_hash: hash, date_activity_started: time(started) },
            primary.map(time), secondary.map(time), transitory.map(time), time(600),
        ).unwrap()
    }

    #[test]
    fn legacy_response_and_missing_transitory_use_character_time() {
        let mut tracker = ActivityTiming::default();
        assert_eq!(sample(&mut tracker, 123, 10, None, None, None).date_activity_started, time(10));
        assert_eq!(sample(&mut tracker, 123, 20, Some(30), Some(30), None).date_activity_started, time(20));
    }

    #[test]
    fn an_offline_profile_can_initialize_with_the_epoch_sentinel() {
        let mut tracker = ActivityTiming::default();
        let epoch = DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap().with_timezone(&Utc);
        let offline = tracker.resolve(
            LatestCharacterActivity { current_activity_hash: 0, date_activity_started: epoch },
            Some(time(20)), Some(time(20)), None, time(30),
        ).unwrap();
        assert_eq!(offline.current_activity_hash, 0);
        assert_eq!(offline.date_activity_started, epoch);
    }

    #[test]
    fn newer_transitory_can_detect_a_same_dungeon_restart_first() {
        let mut tracker = ActivityTiming::default();
        sample(&mut tracker, 123, 10, Some(20), Some(20), Some(10));
        let new_run = sample(&mut tracker, 123, 10, Some(90), Some(90), Some(70));
        assert_eq!(new_run.date_activity_started, time(70));
        assert_eq!(new_run.current_activity_hash, 123);
        // The 204 start catches up without a second timer reset.
        assert_eq!(sample(&mut tracker, 123, 70, Some(100), Some(100), Some(70)), new_run);
    }

    #[test]
    fn new_timer_waits_for_a_character_snapshot_that_can_identify_it() {
        let mut tracker = ActivityTiming::default();
        sample(&mut tracker, 123, 10, Some(20), Some(20), Some(10));
        assert_eq!(sample(&mut tracker, 123, 10, Some(20), Some(90), Some(70)).date_activity_started, time(10));
        let confirmed = sample(&mut tracker, 456, 70, Some(90), Some(90), Some(70));
        assert_eq!(confirmed.current_activity_hash, 456);
        assert_eq!(confirmed.date_activity_started, time(70));
    }

    #[test]
    fn fresh_primary_can_confirm_an_already_received_transitory_start() {
        let mut tracker = ActivityTiming::default();
        sample(&mut tracker, 123, 10, Some(20), Some(90), Some(70));
        assert_eq!(sample(&mut tracker, 123, 10, Some(90), Some(90), Some(70)).date_activity_started, time(70));
    }

    #[test]
    fn old_transitory_does_not_replace_a_new_character_start() {
        let mut tracker = ActivityTiming::default();
        assert_eq!(sample(&mut tracker, 123, 70, Some(90), Some(90), Some(10)).date_activity_started, time(70));
        assert_eq!(sample(&mut tracker, 456, 120, Some(130), Some(130), Some(70)).date_activity_started, time(120));
    }

    #[test]
    fn old_snapshots_cannot_revert_the_timer_or_activity_hash() {
        let mut tracker = ActivityTiming::default();
        let accepted = sample(&mut tracker, 123, 10, Some(90), Some(90), Some(70));
        assert_eq!(sample(&mut tracker, 456, 10, Some(30), Some(30), Some(20)), accepted);
        assert_eq!(sample(&mut tracker, 123, 10, Some(100), Some(100), Some(10)), accepted);
    }

    #[test]
    fn missing_transitory_does_not_reset_a_running_timer() {
        let mut tracker = ActivityTiming::default();
        let accepted = sample(&mut tracker, 123, 10, Some(90), Some(90), Some(70));
        assert_eq!(sample(&mut tracker, 123, 10, Some(100), Some(100), None), accepted);
        assert_eq!(sample(&mut tracker, 123, 120, Some(130), Some(130), None).date_activity_started, time(120));
    }

    #[test]
    fn missing_primary_metadata_does_not_erase_the_freshness_boundary() {
        let mut tracker = ActivityTiming::default();
        sample(&mut tracker, 123, 10, Some(90), Some(90), Some(70));
        let legacy = sample(&mut tracker, 456, 80, None, None, None);
        assert_eq!(legacy.current_activity_hash, 456);
        assert_eq!(legacy.date_activity_started, time(80));
        assert_eq!(sample(&mut tracker, 123, 80, Some(85), Some(85), Some(80)), legacy);
    }

    #[test]
    fn advanced_timer_does_not_block_a_new_hash_with_lagging_character_time() {
        let mut tracker = ActivityTiming::default();
        sample(&mut tracker, 123, 10, Some(90), Some(90), Some(70));
        let changed = sample(&mut tracker, 456, 60, Some(100), Some(100), Some(70));
        assert_eq!(changed.current_activity_hash, 456);
        assert_eq!(changed.date_activity_started, time(70));
    }

    #[test]
    fn orbit_hides_activity_despite_old_transitory_and_can_start_again() {
        let mut tracker = ActivityTiming::default();
        sample(&mut tracker, 123, 10, Some(90), Some(90), Some(70));
        let orbit = sample(&mut tracker, 0, 60, Some(100), Some(100), Some(70));
        assert_eq!(orbit.current_activity_hash, 0);
        assert_eq!(orbit.date_activity_started, time(60));
        let next = sample(&mut tracker, 123, 120, Some(140), Some(140), Some(120));
        assert_eq!(next.current_activity_hash, 123);
        assert_eq!(next.date_activity_started, time(120));
    }

    #[test]
    fn fresh_orbit_is_accepted_even_when_character_start_moves_backwards() {
        let mut tracker = ActivityTiming::default();
        sample(&mut tracker, 123, 70, Some(90), Some(90), Some(70));
        let orbit = sample(&mut tracker, 0, 10, Some(100), Some(95), Some(70));
        assert_eq!(orbit.current_activity_hash, 0);
        assert_eq!(orbit.date_activity_started, time(10));
        // An older active snapshot cannot revive the previous run.
        assert_eq!(sample(&mut tracker, 123, 70, Some(90), Some(95), Some(70)), orbit);
    }

    #[test]
    fn fresh_epoch_orbit_stops_an_active_run_and_allows_the_next_run() {
        let mut tracker = ActivityTiming::default();
        sample(&mut tracker, 123, 70, Some(90), Some(90), Some(70));
        let epoch = DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap().with_timezone(&Utc);
        let orbit = tracker.resolve(
            LatestCharacterActivity { current_activity_hash: 0, date_activity_started: epoch },
            Some(time(100)), Some(time(95)), Some(time(70)), time(600),
        ).unwrap();
        assert_eq!(orbit.current_activity_hash, 0);
        assert_eq!(orbit.date_activity_started, epoch);
        assert_eq!(sample(&mut tracker, 123, 70, Some(90), Some(95), Some(70)), orbit);
        let next = sample(&mut tracker, 123, 120, Some(140), Some(95), Some(70));
        assert_eq!(next.current_activity_hash, 123);
        assert_eq!(next.date_activity_started, time(120));
    }

    #[test]
    fn newer_status_can_switch_activity_despite_an_earlier_start() {
        let mut tracker = ActivityTiming::default();
        sample(&mut tracker, 123, 70, Some(90), None, None);
        let changed = sample(&mut tracker, 456, 60, Some(100), None, None);
        assert_eq!(changed.current_activity_hash, 456);
        assert_eq!(changed.date_activity_started, time(60));
    }

    #[test]
    fn absent_optional_timing_does_not_delay_any_status_transition() {
        let mut tracker = ActivityTiming::default();
        assert_eq!(sample(&mut tracker, 123, 10, Some(20), None, None).current_activity_hash, 123);
        assert_eq!(sample(&mut tracker, 123, 30, Some(40), None, None).date_activity_started, time(30));
        assert_eq!(sample(&mut tracker, 456, 50, Some(60), None, None).current_activity_hash, 456);
        assert_eq!(sample(&mut tracker, 0, 50, Some(70), None, None).current_activity_hash, 0);
    }

    #[test]
    fn orbit_discards_previous_transitory_before_another_activity() {
        let mut tracker = ActivityTiming::default();
        sample(&mut tracker, 123, 10, Some(90), Some(90), Some(70));
        sample(&mut tracker, 0, 10, Some(100), Some(95), Some(70));
        let changed = sample(&mut tracker, 456, 60, Some(110), Some(95), Some(70));
        assert_eq!(changed.current_activity_hash, 456);
        assert_eq!(changed.date_activity_started, time(60));
    }

    #[test]
    fn future_and_unminted_transitory_are_ignored() {
        for (secondary, start) in [(Some(90), Some(700)), (Some(90), Some(95)), (None, Some(70))] {
            let mut tracker = ActivityTiming::default();
            assert_eq!(sample(&mut tracker, 123, 10, Some(100), secondary, start).date_activity_started, time(10));
        }
        let mut tracker = ActivityTiming::default();
        assert_eq!(sample(&mut tracker, 123, 10, None, Some(100), Some(70)).date_activity_started, time(10));
    }

    #[test]
    fn invalid_character_time_cannot_poison_a_good_snapshot() {
        let mut tracker = ActivityTiming::default();
        let accepted = sample(&mut tracker, 123, 10, Some(20), Some(20), Some(10));
        assert_eq!(sample(&mut tracker, 456, 700, Some(30), Some(30), None), accepted);
        // Without a strictly newer status timestamp, keep the legacy guard.
        assert_eq!(sample(&mut tracker, 456, 5, Some(20), Some(40), None), accepted);
        assert_eq!(sample(&mut tracker, 456, 5, None, None, None), accepted);
    }

    #[test]
    fn a_new_profile_does_not_inherit_the_previous_profiles_timing() {
        let mut first = ActivityTiming::default();
        sample(&mut first, 123, 10, Some(200), Some(200), Some(190));
        let mut second = ActivityTiming::default();
        assert_eq!(sample(&mut second, 456, 20, Some(30), Some(30), None).date_activity_started, time(20));
    }
}
