use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};
use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::{
        Shell::SHQueryUserNotificationState,
        WindowsAndMessaging::{
            GetClassNameW, GetForegroundWindow, GetWindowRect, GetWindowTextW, IsIconic,
            IsWindowVisible,
        },
    },
};

use crate::consts::{OVERLAY_POLL_INTERVAL, TARGET_WINDOW_CLASS, TARGET_WINDOW_TITLE};

// Reject failed/truncated reads instead of accepting a partial title or class.
fn matches_text(buffer: &[u16], length: i32, expected: &str) -> bool {
    length > 0
        && (length as usize) < buffer.len().saturating_sub(1)
        && buffer[..length as usize]
            .iter()
            .copied()
            .eq(expected.encode_utf16())
}

fn is_visible_target(hwnd: HWND) -> bool {
    if hwnd.0 == 0
        || !unsafe { IsWindowVisible(hwnd) }.as_bool()
        || unsafe { IsIconic(hwnd) }.as_bool()
    {
        return false;
    }

    // Class first: do not query unrelated window captions (including our own
    // webviews). These fixed buffers never allocate or retain HWND identities.
    let mut class = [0u16; 64];
    let length = unsafe { GetClassNameW(hwnd, &mut class) };
    if !matches_text(&class, length, TARGET_WINDOW_CLASS) {
        return false;
    }

    let mut title = [0u16; 64];
    let length = unsafe { GetWindowTextW(hwnd, &mut title) };
    matches_text(&title, length, TARGET_WINDOW_TITLE)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Bounds {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl Bounds {
    fn from_rect(rect: RECT) -> Option<Self> {
        let width = rect.right.checked_sub(rect.left)?;
        let height = rect.bottom.checked_sub(rect.top)?;
        if width <= 0 || height <= 0 {
            return None;
        }
        Some(Self {
            x: rect.left,
            y: rect.top,
            width: width as u32,
            height: height as u32,
        })
    }
}

// Showing the existing overlay can make it foreground. Preserve that case,
// but treat the previous HWND only as a candidate: all metadata is read again.
fn target_window(foreground: HWND, overlay: HWND, previous: Option<HWND>) -> Option<HWND> {
    if foreground.0 == 0 {
        None
    } else if foreground == overlay {
        previous
    } else {
        Some(foreground)
    }
}

fn foreground_target(overlay: HWND, previous: Option<HWND>) -> Option<(HWND, Bounds)> {
    match unsafe { SHQueryUserNotificationState() } {
        Ok(state) if state.0 != 3 => (),
        // Preserve the exclusive-fullscreen guard; uncertainty hides the overlay.
        _ => return None,
    }

    let foreground = unsafe { GetForegroundWindow() };
    let hwnd = target_window(foreground, overlay, previous)?;
    if !is_visible_target(hwnd) {
        return None;
    }
    let mut rect = RECT::default();
    if !unsafe { GetWindowRect(hwnd, &mut rect) }.as_bool() {
        return None;
    }
    let bounds = Bounds::from_rect(rect)?;
    if unsafe { GetForegroundWindow() } != foreground {
        return None;
    }
    Some((hwnd, bounds))
}

// Remember only successfully applied geometry, never a window's identity.
// A failed/absent target clears this bounded state so re-entry retries fully.
#[derive(Default)]
struct Placement {
    applied: Option<Bounds>,
}

impl Placement {
    fn update(
        &mut self,
        target: Option<Bounds>,
        apply: impl FnOnce(Bounds, bool, bool) -> bool,
    ) -> bool {
        let bounds = match target {
            Some(bounds) => bounds,
            None => {
                self.applied = None;
                return false;
            }
        };
        let position_changed = self
            .applied
            .map_or(true, |old| (old.x, old.y) != (bounds.x, bounds.y));
        let size_changed = self
            .applied
            .map_or(true, |old| (old.width, old.height) != (bounds.width, bounds.height));

        if (position_changed || size_changed) && !apply(bounds, position_changed, size_changed) {
            self.applied = None;
            return false;
        }
        self.applied = Some(bounds);
        true
    }
}

pub async fn overlay_poller(handle: AppHandle) {
    let mut placement = Placement::default();
    let mut previous_target = None;

    loop {
        let overlay = match handle.get_window("overlay") {
            Some(overlay) => overlay,
            None => return,
        };
        // This is our own window handle, never a process handle or PID.
        let overlay_hwnd = overlay.hwnd().ok().map(|hwnd| HWND(hwnd.0));
        let target = overlay_hwnd.and_then(|hwnd| foreground_target(hwnd, previous_target));
        let placed = placement.update(target.map(|(_, bounds)| bounds), |bounds, position, size| {
            if position
                && overlay
                    .set_position(PhysicalPosition {
                        x: bounds.x,
                        y: bounds.y,
                    })
                    .is_err()
            {
                return false;
            }
            if size
                && overlay
                    .set_size(PhysicalSize {
                        width: bounds.width,
                        height: bounds.height,
                    })
                    .is_err()
            {
                return false;
            }
            true
        });

        // Focus or window lifetime can change during placement. A previous HWND
        // never proves identity; validate it again even when our overlay has focus.
        let show = placed
            && target.map_or(false, |(hwnd, _)| {
                overlay_hwnd.and_then(|overlay| {
                    target_window(unsafe { GetForegroundWindow() }, overlay, Some(hwnd))
                }) == Some(hwnd)
                    && is_visible_target(hwnd)
            });
        previous_target = if show {
            target.map(|(hwnd, _)| hwnd)
        } else {
            None
        };
        if !show {
            placement.applied = None;
        }
        // Keep event retries: the frontend serializes visibility operations and
        // skips redundant work, but needs later events to recover from failures.
        let _ = overlay.emit(if show { "show" } else { "hide" }, ());

        tokio::time::sleep(OVERLAY_POLL_INTERVAL).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(value: &str, expected: &str) -> bool {
        let mut buffer = [0u16; 64];
        let value: Vec<_> = value.encode_utf16().collect();
        buffer[..value.len()].copy_from_slice(&value);
        matches_text(&buffer, value.len() as i32, expected)
    }

    fn bounds() -> Bounds {
        Bounds {
            x: 0,
            y: 0,
            width: 2560,
            height: 1440,
        }
    }

    #[test]
    fn observed_game_metadata_matches_but_input_window_does_not() {
        assert!(matches("Destiny 2", TARGET_WINDOW_TITLE));
        assert!(matches("Tiger D3D Window", TARGET_WINDOW_CLASS));
        assert!(!matches("Tiger Input Window", TARGET_WINDOW_CLASS));
        assert!(!matches("Chrome_WidgetWin_1", TARGET_WINDOW_CLASS));
    }

    #[test]
    fn unrelated_or_partial_titles_are_rejected() {
        for title in [
            "",
            "Destiny",
            "Destiny 2 - Browser",
            "Destiny 20",
            "Other Game",
        ] {
            assert!(!matches(title, TARGET_WINDOW_TITLE), "{title}");
        }
    }

    #[test]
    fn failed_truncated_and_invalid_text_reads_are_rejected() {
        let buffer = [b'D' as u16, b'2' as u16, 0];
        for length in [-1, 0, 2, 3, 100] {
            assert!(!matches_text(&buffer, length, "D2"));
        }
        assert!(!matches_text(&[], 1, "D2"));
        assert!(!matches_text(&[0xd800, 0, 0], 1, "D2"));
    }

    #[test]
    fn overlay_focus_requires_a_previous_candidate_and_other_focus_replaces_it() {
        let game = HWND(1);
        let overlay = HWND(2);
        let other = HWND(3);
        assert_eq!(target_window(game, overlay, None), Some(game));
        assert_eq!(target_window(overlay, overlay, Some(game)), Some(game));
        assert_eq!(target_window(overlay, overlay, None), None);
        assert_eq!(target_window(other, overlay, Some(game)), Some(other));
        assert_eq!(target_window(HWND(0), overlay, Some(game)), None);
    }

    #[test]
    fn physical_geometry_preserves_negative_monitor_coordinates() {
        assert_eq!(
            Bounds::from_rect(RECT {
                left: -2560,
                top: -120,
                right: 0,
                bottom: 1320,
            }),
            Some(Bounds {
                x: -2560,
                y: -120,
                ..bounds()
            })
        );
    }

    #[test]
    fn empty_inverted_and_overflowing_geometry_is_rejected() {
        for rect in [
            RECT::default(),
            RECT {
                left: 2,
                top: 0,
                right: 1,
                bottom: 10,
            },
            RECT {
                left: 0,
                top: 2,
                right: 10,
                bottom: 1,
            },
            RECT {
                left: i32::MIN,
                top: 0,
                right: i32::MAX,
                bottom: 10,
            },
        ] {
            assert_eq!(Bounds::from_rect(rect), None);
        }
    }

    #[test]
    fn unchanged_geometry_does_not_update_the_window() {
        let mut placement = Placement::default();
        assert!(placement.update(Some(bounds()), |_, position, size| position && size));
        assert!(placement.update(Some(bounds()), |_, _, _| panic!("redundant update")));
    }

    #[test]
    fn movement_and_resize_only_update_the_changed_dimension() {
        let mut placement = Placement::default();
        assert!(placement.update(Some(bounds()), |_, _, _| true));
        let moved = Bounds {
            x: -2560,
            ..bounds()
        };
        assert!(placement.update(Some(moved), |_, position, size| position && !size));
        let resized = Bounds {
            width: 1920,
            ..moved
        };
        assert!(placement.update(Some(resized), |_, position, size| !position && size));
    }

    #[test]
    fn lost_target_hides_and_reentry_reapplies_geometry() {
        let mut placement = Placement::default();
        assert!(placement.update(Some(bounds()), |_, _, _| true));
        assert!(!placement.update(None, |_, _, _| panic!("missing target")));
        assert_eq!(placement.applied, None);
        assert!(placement.update(Some(bounds()), |_, position, size| position && size));
    }

    #[test]
    fn failed_placement_hides_and_retries_in_full() {
        let mut placement = Placement::default();
        assert!(placement.update(Some(bounds()), |_, _, _| true));
        let moved = Bounds { x: 1, ..bounds() };
        assert!(!placement.update(Some(moved), |_, _, _| false));
        assert_eq!(placement.applied, None);
        assert!(placement.update(Some(moved), |_, position, size| position && size));
    }
}
