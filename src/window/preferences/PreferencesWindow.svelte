<script lang="ts">
    import { appWindow } from "@tauri-apps/api/window";
    import LineButton from "../widgets/LineButton.svelte";
    import StyledCheckbox from "./StyledCheckbox.svelte";
    import type { Preferences } from "../../core/types";
    import * as ipc from "../../core/ipc";

    let preferences: Preferences;
    let error: string;

    function init() {
        ipc.getPreferences().then((p: Preferences) => (preferences = p));
    }

    function confirm() {
        ipc.setPreferences(preferences)
            .then(() => appWindow.close())
            .catch((e) => {
                error = e.message ?? e;
                appWindow.show();
            });

        appWindow.hide();
    }

    init();
</script>

<main>
    <h1>Preferences</h1>
    {#if preferences}
        <div class="preferences">
            {#if error}
                <p class="error">{error}</p>
            {/if}

            <div class="preference">
                <StyledCheckbox bind:checked={preferences.enableOverlay}>Enable overlay</StyledCheckbox>
            </div>

            <div class="preference-group">
                <div class="preference">
                    <StyledCheckbox
                        bind:checked={preferences.displayDailyClears}
                        disabled={!preferences.enableOverlay}
                        >Display activity clears</StyledCheckbox
                    >
                </div>
                <div class="preference">
                    <StyledCheckbox
                        bind:checked={preferences.displayClearNotifications}
                        disabled={!preferences.enableOverlay}
                        >Display activity clear notifications</StyledCheckbox
                    >
                </div>
                <div class="preference">
                    <StyledCheckbox
                        bind:checked={preferences.displayMilliseconds}
                        disabled={!preferences.enableOverlay || !preferences.displayTimer}
                        >Display timer milliseconds</StyledCheckbox
                    >
                </div>
            </div>

            <h2>Custom features</h2>
            <div class="preference-group custom-group">
                <div class="preference">
                    <StyledCheckbox
                        bind:checked={preferences.displayTimer}
                        disabled={!preferences.enableOverlay}
                        >Display timer</StyledCheckbox
                    >
                </div>
                <div class="preference">
                    <StyledCheckbox
                        bind:checked={preferences.displayActivityName}
                        disabled={!preferences.enableOverlay}
                        >Display activity name</StyledCheckbox
                    >
                </div>
                <div class="preference">
                    <StyledCheckbox
                        bind:checked={preferences.autoHideActivityName}
                        disabled={!preferences.enableOverlay || !preferences.displayActivityName}
                        >Automatically hide activity name</StyledCheckbox
                    >
                </div>
                <div class="preference number-preference">
                    <label for="activity-name-delay">Hide after</label>
                    <input
                        id="activity-name-delay"
                        type="number"
                        min="0"
                        max="3600"
                        step="1"
                        bind:value={preferences.activityNameHideDelaySeconds}
                        disabled={!preferences.enableOverlay || !preferences.displayActivityName || !preferences.autoHideActivityName}
                    />
                    <span>seconds</span>
                </div>
            </div>

            <div class="actions">
                <LineButton clickCallback={confirm}>Confirm</LineButton>
            </div>
        </div>
    {/if}
</main>

<style>
    h1 {
        margin: 24px 48px;
    }

    h2 {
        margin: 28px 48px 12px;
    }

    .preferences {
        margin: 16px 48px;
    }

    .preference-group {
        padding: 8px 12px;
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .custom-group {
        margin-top: 0;
    }

    .preference {
        margin: 12px 8px;
    }

    .number-preference {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .number-preference input {
        width: 70px;
        padding: 4px 6px;
        color: inherit;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 3px;
    }

    .error {
        color: var(--error);
    }

    .actions {
        margin-top: 24px;
        float: right;
    }
</style>
