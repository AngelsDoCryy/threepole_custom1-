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
    pub display_activity_name: bool,
    pub auto_hide_activity_name: bool,
    pub activity_name_hide_delay_seconds: u32,
    pub display_activity_icon: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            enable_overlay: false,
            display_daily_clears: true,
            display_clear_notifications: true,
            display_milliseconds: true,
            display_activity_name: true,
            auto_hide_activity_name: true,
            activity_name_hide_delay_seconds: 10,
            display_activity_icon: true,
        }
    }
}

impl ConfigFile for Preferences {
    fn get_filename() -> &'static str {
        "preferences.json"
    }
}
