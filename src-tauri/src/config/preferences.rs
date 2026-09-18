use serde::{Deserialize, Serialize};

use super::ConfigFile;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Preferences {
    pub enable_overlay: bool,
    pub display_daily_clears: bool,
    pub display_clear_notifications: bool,
    pub display_milliseconds: bool,
    pub display_timer: bool,
    pub display_activity_name: bool,
    pub auto_hide_activity_name: bool,
    pub activity_name_hide_delay_seconds: u32,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            enable_overlay: false,
            display_daily_clears: true,
            display_clear_notifications: true,
            display_milliseconds: true,
            display_timer: true,
            display_activity_name: true,
            auto_hide_activity_name: true,
            activity_name_hide_delay_seconds: 10,
        }
    }
}

impl ConfigFile for Preferences {
    fn get_filename() -> &'static str {
        "preferences.json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_preferences_keep_timer_enabled() {
        let p: Preferences = serde_json::from_str(r#"{"enableOverlay":true,"displayDailyClears":false}"#).unwrap();
        assert!(p.display_timer);
        assert!(!p.display_daily_clears);
    }

    #[test]
    fn notifications_only_preferences_round_trip() {
        let p: Preferences = serde_json::from_str(r#"{"enableOverlay":true,"displayTimer":false,"displayDailyClears":false,"displayActivityName":false,"displayClearNotifications":true}"#).unwrap();
        let saved = serde_json::to_string(&p).unwrap();
        let restored: Preferences = serde_json::from_str(&saved).unwrap();
        assert!(!restored.display_timer);
        assert!(!restored.display_daily_clears);
        assert!(!restored.display_activity_name);
        assert!(restored.display_clear_notifications);
    }
}
