//! Capability routing only: standard mode never opens CDP or repairs launchers.
use crate::models::{MediaFeature, MediaState, NeteaseFeatures, NeteaseProgressMode};
use std::time::{Duration, Instant};

fn feature(source: &str, readable: bool, controllable: bool) -> MediaFeature {
    MediaFeature { source: source.into(), readable, controllable }
}

pub(crate) fn route(snapshot: &mut MediaState, enhanced: bool, connected: bool, mode: NeteaseProgressMode) {
    if snapshot.source != "netease" { snapshot.netease_features = None; return; }
    let cdp = enhanced && connected;
    let cdp_progress = cdp && snapshot.netease_progress_available == Some(true) && snapshot.progress_known;
    let progress_source = if !snapshot.progress_known { "unavailable" }
        else if mode == NeteaseProgressMode::Estimate { "estimate" }
        else if cdp_progress { "cdp" } else { "smtc" };
    if cdp { snapshot.capabilities.seek = true; }
    snapshot.netease_features = Some(NeteaseFeatures {
        progress: feature(progress_source, snapshot.progress_known, snapshot.capabilities.seek && snapshot.progress_known),
        like: feature(if cdp { "cdp" } else { "unavailable" }, cdp, cdp),
        mode: feature(if cdp { "cdp" } else { "smtc" }, cdp || snapshot.shuffle_active.is_some() || snapshot.repeat_mode.is_some(),
            cdp || snapshot.capabilities.shuffle || snapshot.capabilities.repeat),
    });
}

/// A failed progress sample invalidates only the progress anchor. Keep the
/// independent mode/like connection when the player is loading or the song
/// identity changed; drop all CDP features only for an actual connection loss.
pub(crate) fn progress_unavailable(snapshot: &mut MediaState, enhanced: bool, connected: bool, mode: NeteaseProgressMode) {
    snapshot.netease_progress_available = Some(false);
    snapshot.progress_known = false;
    snapshot.capabilities.seek = false;
    route(snapshot, enhanced, connected, mode);
    snapshot.capabilities.seek = false;
}

/// A frozen SMTC value is not a real timeline. Require movement while playing,
/// and expire evidence after three seconds. Paused, valid snapshots are usable.
#[derive(Default)]
pub(crate) struct TimelineEvidence {
    last_position: Option<u64>,
    last_movement: Option<Instant>,
}
impl TimelineEvidence {
    pub(crate) fn observe(&mut self, position: u64, duration: u64, playing: bool, now: Instant) -> bool {
        if duration == 0 || position > duration { *self = Self::default(); return false; }
        if self.last_position.is_some_and(|last| last != position) {
            self.last_movement = Some(now);
        }
        self.last_position = Some(position);
        !playing || self.last_movement.is_some_and(|last| now.saturating_duration_since(last) <= Duration::from_secs(3))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn track_transition_drops_progress_without_switching_mode_or_like_to_smtc() {
        let mut state = MediaState { source: "netease".into(), progress_known: true, netease_progress_available: Some(true), ..Default::default() };
        state.capabilities.shuffle = true;
        state.shuffle_active = Some(true);
        route(&mut state, true, true, NeteaseProgressMode::Real);
        progress_unavailable(&mut state, true, true, NeteaseProgressMode::Real);
        let features = state.netease_features.as_ref().unwrap();
        assert_eq!(features.mode.source, "cdp");
        assert!(features.mode.controllable && features.like.controllable);
        assert!(!features.progress.readable && !features.progress.controllable);
        assert!(!state.capabilities.seek);
        progress_unavailable(&mut state, true, false, NeteaseProgressMode::Real);
        let features = state.netease_features.unwrap();
        assert_eq!(features.mode.source, "smtc");
        assert!(!features.like.controllable);
    }
    #[test]
    fn standard_and_disconnected_routes_never_claim_cdp_capabilities() {
        for (enhanced, connected) in [(false, false), (false, true), (true, false)] {
            let mut state = MediaState { source: "netease".into(), progress_known: false, ..Default::default() };
            route(&mut state, enhanced, connected, NeteaseProgressMode::Real);
            let features = state.netease_features.unwrap();
            assert!(!features.like.readable && !features.like.controllable);
            assert_eq!(features.progress.source, "unavailable");
            assert!(!state.capabilities.seek);
        }
    }
    #[test]
    fn enhanced_and_partial_system_capabilities_are_independent() {
        let mut state = MediaState { source: "netease".into(), progress_known: true, ..Default::default() };
        state.capabilities.shuffle = true;
        state.shuffle_active = Some(false);
        route(&mut state, false, false, NeteaseProgressMode::Real);
        assert!(state.netease_features.as_ref().unwrap().mode.controllable);
        assert!(!state.netease_features.as_ref().unwrap().like.readable);
        state.netease_progress_available = Some(true);
        route(&mut state, true, true, NeteaseProgressMode::Real);
        assert_eq!(state.netease_features.as_ref().unwrap().progress.source, "cdp");
        assert!(state.netease_features.as_ref().unwrap().like.controllable);
        route(&mut state, true, true, NeteaseProgressMode::Estimate);
        assert_eq!(state.netease_features.as_ref().unwrap().progress.source, "estimate");
    }
    #[test]
    fn frozen_timeline_expires_and_recovers_on_movement() {
        let now = Instant::now();
        let mut evidence = TimelineEvidence::default();
        assert!(!evidence.observe(0, 0, true, now));
        assert!(!evidence.observe(10_000, 100_000, true, now));
        assert!(evidence.observe(11_000, 100_000, true, now + Duration::from_secs(1)));
        assert!(!evidence.observe(11_000, 100_000, true, now + Duration::from_secs(5)));
        assert!(evidence.observe(12_000, 100_000, true, now + Duration::from_secs(6)));
        assert!(evidence.observe(12_000, 100_000, false, now + Duration::from_secs(9)));
    }
}
