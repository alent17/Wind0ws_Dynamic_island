use crate::models::MediaState;
use crate::models::NeteaseProgressMode;
use crate::services::netease_cdp::{CdpError, NeteaseProgress};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::time::Duration;

pub(crate) const MEDIA_ACTOR_QUEUE_CAPACITY: usize = 8;
const MEDIA_ACTOR_REPLY_TIMEOUT: Duration = Duration::from_secs(4);
pub(crate) const MAX_SAFE_NATIVE_REVISION: u64 = 9_007_199_254_740_991;

pub(crate) enum MediaActorMessage {
    Refresh,
    Snapshot(SyncSender<Result<MediaState, String>>),
    TransportChanged,
    VerifiedNeteaseSeek {
        expected_epoch: String,
        expected_title: String,
        expected_artist: String,
        expected_track_generation: u64,
        position_ms: u64,
        duration_ms: u64,
        track_id: String,
        play_id: String,
        reply: SyncSender<Result<MediaState, String>>,
    },
    NeteaseProgressReadCompleted {
        expected_epoch: String,
        expected_title: String,
        expected_artist: String,
        expected_track_generation: u64,
        expected_seek_generation: u64,
        started_native_revision: u64,
        result: Result<NeteaseProgress, CdpError>,
    },
    NeteaseProgressModeChanged,
    PlayerIdentityChanged,
    #[cfg(feature = "c2-media-probe")]
    Stop,
}

pub(crate) struct NativeSnapshotCache {
    epoch: String,
    revision: u64,
    track_generation: u64,
    snapshot: Option<MediaState>,
}

impl NativeSnapshotCache {
    pub(crate) fn new(epoch: String) -> Self {
        Self {
            epoch,
            revision: 0,
            track_generation: 0,
            snapshot: None,
        }
    }

    pub(crate) fn current(&self) -> Option<MediaState> {
        self.snapshot.clone()
    }

    pub(crate) fn publish(&mut self, mut snapshot: MediaState) -> Result<MediaState, String> {
        let identity_changed = self.snapshot.as_ref().is_none_or(|current| {
            current.source != snapshot.source
                || current.source_display != snapshot.source_display
                || current.title != snapshot.title
                || current.artist != snapshot.artist
        });
        if identity_changed {
            self.track_generation = self
                .track_generation
                .checked_add(1)
                .ok_or_else(|| "native media track generation exhausted".to_string())?;
        }
        let next = self
            .revision
            .checked_add(1)
            .filter(|revision| *revision <= MAX_SAFE_NATIVE_REVISION)
            .ok_or_else(|| {
                "native media revision exhausted JavaScript safe-integer range".to_string()
            })?;
        snapshot.native_epoch = self.epoch.clone();
        snapshot.native_revision = next;
        snapshot.native_track_generation = self.track_generation;
        self.revision = next;
        self.snapshot = Some(snapshot.clone());
        Ok(snapshot)
    }

    pub(crate) fn publish_verified_netease_seek(
        &mut self,
        expected_epoch: &str,
        expected_title: &str,
        expected_artist: &str,
        expected_track_generation: u64,
        track_id: &str,
        play_id: &str,
        position_ms: u64,
        duration_ms: u64,
        mode: NeteaseProgressMode,
    ) -> Result<MediaState, String> {
        if expected_epoch != self.epoch
            || track_id.is_empty()
            || play_id.is_empty()
            || duration_ms == 0
            || position_ms >= duration_ms
        {
            return Err("verified NetEase seek has an invalid basis or timeline".to_string());
        }
        let mut snapshot = self
            .snapshot
            .clone()
            .ok_or_else(|| "media actor has no current snapshot".to_string())?;
        validate_netease_seek_basis(
            &snapshot,
            expected_epoch,
            expected_title,
            expected_artist,
            expected_track_generation,
        )?;
        snapshot.position_ms = position_ms;
        snapshot.duration_ms = duration_ms;
        snapshot.progress_known = true;
        snapshot.netease_progress_available = Some(true);
        crate::services::netease::route(&mut snapshot, true, true, mode);
        snapshot.last_updated_timestamp = now_epoch_ms();
        self.publish(snapshot)
    }

    pub(crate) fn publish_netease_progress(
        &mut self,
        expected_epoch: &str,
        expected_title: &str,
        expected_artist: &str,
        expected_track_generation: u64,
        progress: &NeteaseProgress,
    ) -> Result<MediaState, String> {
        if progress.track_id.is_empty()
            || progress.play_id.is_empty()
            || progress.position_ms > progress.duration_ms
            || progress.duration_ms == 0
        {
            return Err("NetEase progress sample has an invalid identity or timeline".to_string());
        }
        if progress.title != expected_title || progress.artist != expected_artist {
            return Err("NetEase progress sample belongs to a different track".to_string());
        }
        let mut snapshot = self
            .snapshot
            .clone()
            .ok_or_else(|| "media actor has no current snapshot".to_string())?;
        validate_netease_seek_basis(
            &snapshot,
            expected_epoch,
            expected_title,
            expected_artist,
            expected_track_generation,
        )?;
        snapshot.position_ms = progress.position_ms;
        snapshot.duration_ms = progress.duration_ms;
        snapshot.progress_known = true;
        snapshot.netease_progress_available = Some(true);
        crate::services::netease::route(&mut snapshot, true, true, NeteaseProgressMode::Real);
        snapshot.last_updated_timestamp = now_epoch_ms();
        self.publish(snapshot)
    }
}

fn now_epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Clone, Debug, Default)]
pub(crate) struct NeteaseProgressScheduler {
    last_success: Option<std::time::Instant>,
    retry_after: Option<std::time::Instant>,
    consecutive_failures: u8,
}

impl NeteaseProgressScheduler {
    pub(crate) fn due(&self, now: std::time::Instant, selected: bool, playing: bool) -> bool {
        self.next_due(now, selected, playing)
            .is_some_and(|due| now >= due)
    }

    pub(crate) fn next_due(
        &self,
        now: std::time::Instant,
        selected: bool,
        playing: bool,
    ) -> Option<std::time::Instant> {
        if !selected {
            return None;
        }
        let ttl = if playing {
            Duration::from_millis(200)
        } else {
            Duration::from_millis(500)
        };
        let due = self.last_success.map_or(now, |success| success + ttl);
        Some(self.retry_after.map_or(due, |retry| due.max(retry)))
    }

    pub(crate) fn retry_soon(&mut self, now: std::time::Instant) {
        self.retry_after = Some(now + Duration::from_millis(100));
    }

    pub(crate) fn success(&mut self, now: std::time::Instant) {
        self.last_success = Some(now);
        self.retry_after = None;
        self.consecutive_failures = 0;
    }

    pub(crate) fn failure(&mut self, now: std::time::Instant) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1).min(6);
        let delay_ms = (250u64 << (self.consecutive_failures - 1)).min(5_000);
        self.retry_after = Some(now + Duration::from_millis(delay_ms));
    }

    pub(crate) fn reset(&mut self) {
        self.last_success = None;
        self.retry_after = None;
        self.consecutive_failures = 0;
    }

    pub(crate) fn mode_enabled(&self, mode: NeteaseProgressMode) -> bool {
        mode == NeteaseProgressMode::Real
    }
}

pub(crate) fn reconcile_netease_progress_mode(
    current: &mut NeteaseProgressMode,
    desired: NeteaseProgressMode,
    seek_generation: &mut u64,
    scheduler: &mut NeteaseProgressScheduler,
) -> bool {
    if *current == desired {
        return false;
    }
    *current = desired;
    *seek_generation = seek_generation.saturating_add(1);
    if desired == NeteaseProgressMode::Real {
        scheduler.reset();
    }
    true
}

#[derive(Clone, Debug)]
pub(crate) struct NeteaseTimelineSample {
    epoch: String,
    track_generation: u64,
    title: String,
    artist: String,
    track_id: String,
    play_id: String,
    position_ms: u64,
    duration_ms: u64,
    sampled_at: std::time::Instant,
    sampled_at_epoch_ms: u64,
    is_playing: bool,
}

impl NeteaseTimelineSample {
    pub(crate) fn from_progress(
        snapshot: &MediaState,
        progress: &NeteaseProgress,
        sampled_at: std::time::Instant,
    ) -> Self {
        Self {
            epoch: snapshot.native_epoch.clone(),
            track_generation: snapshot.native_track_generation,
            title: snapshot.title.clone(),
            artist: snapshot.artist.clone(),
            track_id: progress.track_id.clone(),
            play_id: progress.play_id.clone(),
            position_ms: progress.position_ms,
            duration_ms: progress.duration_ms,
            sampled_at,
            sampled_at_epoch_ms: now_epoch_ms(),
            is_playing: snapshot.is_playing,
        }
    }

    pub(crate) fn apply(&mut self, snapshot: &mut MediaState, now: std::time::Instant) -> bool {
        if snapshot.source != "netease"
            || snapshot.native_epoch != self.epoch
            || snapshot.native_track_generation != self.track_generation
            || snapshot.title != self.title
            || snapshot.artist != self.artist
        {
            return false;
        }
        if self.is_playing != snapshot.is_playing {
            if self.is_playing {
                self.position_ms = self
                    .position_ms
                    .saturating_add(
                        now.saturating_duration_since(self.sampled_at).as_millis() as u64
                    )
                    .min(self.duration_ms);
            }
            self.sampled_at = now;
            self.sampled_at_epoch_ms = now_epoch_ms();
            self.is_playing = snapshot.is_playing;
        }
        snapshot.position_ms = self.position_ms;
        snapshot.duration_ms = self.duration_ms;
        snapshot.last_updated_timestamp = self.sampled_at_epoch_ms;
        true
    }

    #[cfg(test)]
    fn cdp_identity(&self) -> (&str, &str) {
        (&self.track_id, &self.play_id)
    }
}

pub(crate) fn validate_netease_seek_basis(
    snapshot: &MediaState,
    expected_epoch: &str,
    expected_title: &str,
    expected_artist: &str,
    expected_track_generation: u64,
) -> Result<(), String> {
    if snapshot.native_epoch != expected_epoch
        || snapshot.source != "netease"
        || snapshot.title != expected_title
        || snapshot.artist != expected_artist
        || snapshot.native_track_generation != expected_track_generation
    {
        return Err("media actor track changed before verified seek synchronization".to_string());
    }
    Ok(())
}

pub(crate) fn run_if_netease_seek_basis<T>(
    snapshot: &MediaState,
    expected_epoch: &str,
    expected_title: &str,
    expected_artist: &str,
    expected_track_generation: u64,
    action: impl FnOnce() -> T,
) -> Result<T, String> {
    validate_netease_seek_basis(
        snapshot,
        expected_epoch,
        expected_title,
        expected_artist,
        expected_track_generation,
    )?;
    Ok(action())
}

pub(crate) fn progress_read_basis_is_current(
    snapshot: &MediaState,
    expected_epoch: &str,
    expected_title: &str,
    expected_artist: &str,
    expected_track_generation: u64,
    expected_seek_generation: u64,
    current_seek_generation: u64,
    started_native_revision: u64,
) -> bool {
    current_seek_generation == expected_seek_generation
        && snapshot.native_revision >= started_native_revision
        && validate_netease_seek_basis(
            snapshot,
            expected_epoch,
            expected_title,
            expected_artist,
            expected_track_generation,
        )
        .is_ok()
}

#[derive(Clone, Default)]
pub(crate) struct NeteaseProgressInFlight(std::sync::Arc<std::sync::atomic::AtomicBool>);

pub(crate) struct NeteaseProgressPermit(std::sync::Arc<std::sync::atomic::AtomicBool>);

impl Drop for NeteaseProgressPermit {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::Release);
    }
}

impl NeteaseProgressInFlight {
    pub(crate) fn begin(&self) -> Option<NeteaseProgressPermit> {
        self.0
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            )
            .ok()
            .map(|_| NeteaseProgressPermit(self.0.clone()))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NeteaseSeekAnchor {
    title: String,
    artist: String,
    position_ms: u64,
    duration_ms: u64,
    is_playing: bool,
    started_at: std::time::Instant,
    updated_at: std::time::Instant,
}

impl NeteaseSeekAnchor {
    const MAX_LIFETIME: Duration = Duration::from_secs(5);
    pub(crate) fn from_snapshot(snapshot: &MediaState) -> Self {
        Self {
            title: snapshot.title.clone(),
            artist: snapshot.artist.clone(),
            position_ms: snapshot.position_ms,
            duration_ms: snapshot.duration_ms,
            is_playing: snapshot.is_playing,
            started_at: std::time::Instant::now(),
            updated_at: std::time::Instant::now(),
        }
    }

    pub(crate) fn apply(
        &mut self,
        snapshot: &mut MediaState,
        now: std::time::Instant,
        estimate_mode: bool,
    ) -> bool {
        if !estimate_mode && now.saturating_duration_since(self.started_at) > Self::MAX_LIFETIME {
            return false;
        }
        if snapshot.source != "netease"
            || snapshot.title != self.title
            || snapshot.artist != self.artist
        {
            return false;
        }
        if self.is_playing {
            self.position_ms = self
                .position_ms
                .saturating_add(now.saturating_duration_since(self.updated_at).as_millis() as u64)
                .min(self.duration_ms);
        }
        self.updated_at = now;
        self.is_playing = snapshot.is_playing;
        snapshot.position_ms = self.position_ms;
        snapshot.duration_ms = self.duration_ms;
        snapshot.progress_known = true;
        true
    }
}

pub(crate) fn queue_snapshot_request(
    cache: &NativeSnapshotCache,
    reply: SyncSender<Result<MediaState, String>>,
    pending: &mut Vec<SyncSender<Result<MediaState, String>>>,
) -> bool {
    if let Some(snapshot) = cache.current() {
        let _ = reply.send(Ok(snapshot));
        false
    } else {
        let should_refresh = pending.is_empty();
        pending.push(reply);
        should_refresh
    }
}

pub(crate) fn complete_snapshot_requests(
    snapshot: &MediaState,
    pending_snapshots: &mut Vec<SyncSender<Result<MediaState, String>>>,
) {
    for reply in pending_snapshots.drain(..) {
        let _ = reply.send(Ok(snapshot.clone()));
    }
}

#[derive(Clone)]
pub(crate) struct MediaActorHandle {
    sender: SyncSender<MediaActorMessage>,
}

impl MediaActorHandle {
    pub(crate) fn new(sender: SyncSender<MediaActorMessage>) -> Self {
        Self { sender }
    }

    #[cfg(feature = "c2-media-probe")]
    pub(crate) fn stop(&self) {
        let _ = self.sender.send(MediaActorMessage::Stop);
    }

    pub(crate) async fn snapshot(&self) -> Result<MediaState, String> {
        let (reply, response) = mpsc::sync_channel(1);
        self.try_request(MediaActorMessage::Snapshot(reply))?;
        await_response(response, "media snapshot").await
    }

    pub(crate) fn wake_for_transport_change(&self) {
        let _ = self.sender.try_send(MediaActorMessage::TransportChanged);
    }

    pub(crate) fn wake_for_netease_progress_mode_change(&self) {
        // Settings are authoritative shared state. This bounded message only
        // wakes the actor; saturation cannot lose the latest saved mode.
        let _ = self
            .sender
            .try_send(MediaActorMessage::NeteaseProgressModeChanged);
    }

    pub(crate) fn wake_for_player_identity_change(&self) {
        let _ = self
            .sender
            .try_send(MediaActorMessage::PlayerIdentityChanged);
    }

    pub(crate) async fn confirmed_netease_seek(
        &self,
        expected_epoch: String,
        expected_title: String,
        expected_artist: String,
        expected_track_generation: u64,
        track_id: String,
        play_id: String,
        position_ms: u64,
        duration_ms: u64,
    ) -> Result<MediaState, String> {
        let (reply, response) = mpsc::sync_channel(1);
        self.try_request(MediaActorMessage::VerifiedNeteaseSeek {
            expected_epoch,
            expected_title,
            expected_artist,
            expected_track_generation,
            track_id,
            play_id,
            position_ms,
            duration_ms,
            reply,
        })?;
        await_response(response, "verified NetEase seek synchronization").await
    }

    fn try_request(&self, message: MediaActorMessage) -> Result<(), String> {
        self.sender.try_send(message).map_err(|error| match error {
            TrySendError::Full(_) => "media actor request queue is full".to_string(),
            TrySendError::Disconnected(_) => "media actor is unavailable".to_string(),
        })
    }
}

async fn await_response<T: Send + 'static>(
    response: Receiver<Result<T, String>>,
    operation: &'static str,
) -> Result<T, String> {
    await_response_timeout(response, operation, MEDIA_ACTOR_REPLY_TIMEOUT).await
}

async fn await_response_timeout<T: Send + 'static>(
    response: Receiver<Result<T, String>>,
    operation: &'static str,
    timeout: Duration,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(move || response.recv_timeout(timeout))
        .await
        .map_err(|error| format!("{operation} worker failed: {error}"))?
        .map_err(|error| match error {
            mpsc::RecvTimeoutError::Timeout => format!("{operation} timed out"),
            mpsc::RecvTimeoutError::Disconnected => format!("{operation} actor stopped"),
        })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publishes_full_actor_snapshots_in_one_epoch_with_safe_monotonic_revisions() {
        let mut cache = NativeSnapshotCache::new("epoch-1".into());
        let first = cache
            .publish(MediaState {
                album_art: "full-cover".into(),
                ..MediaState::default()
            })
            .unwrap();
        let second = cache
            .publish(MediaState {
                album_art: "".into(),
                ..MediaState::default()
            })
            .unwrap();
        assert_eq!(
            (first.native_epoch.as_str(), first.native_revision),
            ("epoch-1", 1)
        );
        assert_eq!(
            (second.native_epoch.as_str(), second.native_revision),
            ("epoch-1", 2)
        );
        assert_eq!(cache.current().unwrap().album_art, "");
    }

    #[test]
    fn snapshot_requests_wait_for_actor_refresh_then_reuse_the_published_revision() {
        let mut cache = NativeSnapshotCache::new("epoch-1".into());
        let mut pending = Vec::new();
        let (reply, response) = mpsc::sync_channel(1);
        assert!(queue_snapshot_request(&cache, reply, &mut pending));
        assert_eq!(pending.len(), 1);

        let published = cache
            .publish(MediaState {
                album_art: "full".into(),
                ..MediaState::default()
            })
            .unwrap();
        complete_snapshot_requests(&published, &mut pending);
        let response = response.try_recv().unwrap().unwrap();
        assert_eq!(response.native_revision, published.native_revision);
        assert_eq!(response.album_art, "full");

        let (cached_reply, cached_response) = mpsc::sync_channel(1);
        assert!(!queue_snapshot_request(&cache, cached_reply, &mut pending));
        assert_eq!(
            cached_response.try_recv().unwrap().unwrap().native_revision,
            published.native_revision
        );
        assert_eq!(
            cache.current().unwrap().native_revision,
            published.native_revision
        );
    }

    #[test]
    fn transport_completion_requests_refresh_without_waiting_for_metadata_confirmation() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let actor = MediaActorHandle::new(sender);
        actor.wake_for_transport_change();
        assert!(matches!(
            receiver.try_recv(),
            Ok(MediaActorMessage::TransportChanged)
        ));
    }

    #[test]
    fn snapshot_confirmation_completes_from_one_published_snapshot() {
        let mut cache = NativeSnapshotCache::new("epoch-1".into());
        let (reply, response) = mpsc::sync_channel(1);
        let mut pending_snapshots = vec![reply];
        let published = cache
            .publish(MediaState {
                is_playing: true,
                ..MediaState::default()
            })
            .unwrap();
        complete_snapshot_requests(&published, &mut pending_snapshots);
        assert_eq!(cache.current().unwrap().native_revision, 1);
        assert!(response.try_recv().unwrap().is_ok());
    }

    #[test]
    fn revision_never_wraps_or_exceeds_javascript_safe_integer() {
        let mut cache = NativeSnapshotCache::new("epoch-1".into());
        cache.revision = MAX_SAFE_NATIVE_REVISION;
        assert!(cache
            .publish(MediaState::default())
            .err()
            .unwrap()
            .contains("safe-integer"));
        assert_eq!(cache.revision, MAX_SAFE_NATIVE_REVISION);
    }

    #[test]
    fn verified_netease_seek_publishes_only_for_the_current_stamped_track() {
        let mut cache = NativeSnapshotCache::new("epoch-1".into());
        cache
            .publish(MediaState {
                source: "netease".into(),
                title: "Song".into(),
                artist: "Artist".into(),
                position_ms: 0,
                duration_ms: 180_000,
                ..MediaState::default()
            })
            .unwrap();
        let generation = cache.current().unwrap().native_track_generation;
        let snapshot = cache
            .publish_verified_netease_seek(
                "epoch-1", "Song", "Artist", generation, "track-1", "play-1", 42_000, 180_000, NeteaseProgressMode::Estimate,
            )
            .unwrap();
        assert_eq!(snapshot.position_ms, 42_000);
        assert_eq!(snapshot.netease_features.as_ref().unwrap().progress.source, "estimate");
        assert_eq!(snapshot.native_revision, 2);
        assert!(cache
            .publish_verified_netease_seek(
                "epoch-1",
                "Different",
                "Artist",
                generation,
                "track-1",
                "play-1",
                50_000,
                180_000,
                NeteaseProgressMode::Real,
            )
            .is_err());
        assert!(cache
            .publish_verified_netease_seek(
                "other-epoch",
                "Song",
                "Artist",
                generation,
                "track-1",
                "play-1",
                50_000,
                180_000,
                NeteaseProgressMode::Real,
            )
            .is_err());
        assert_eq!(cache.current().unwrap().position_ms, 42_000);
    }

    #[test]
    fn actor_basis_is_validated_before_external_seek_action() {
        let mut cache = NativeSnapshotCache::new("epoch-1".into());
        let snapshot = cache
            .publish(MediaState {
                source: "netease".into(),
                title: "Song".into(),
                artist: "Artist".into(),
                ..MediaState::default()
            })
            .unwrap();
        let calls = std::cell::Cell::new(0);
        let rejected = run_if_netease_seek_basis(
            &snapshot,
            "epoch-1",
            "Other",
            "Artist",
            snapshot.native_track_generation,
            || calls.set(calls.get() + 1),
        );
        assert!(rejected.is_err());
        assert_eq!(calls.get(), 0);
        let accepted = run_if_netease_seek_basis(
            &snapshot,
            "epoch-1",
            "Song",
            "Artist",
            snapshot.native_track_generation,
            || calls.set(calls.get() + 1),
        );
        assert!(accepted.is_ok());
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn progress_read_basis_drops_track_and_seek_stale_results_but_allows_newer_poll_revisions() {
        let mut cache = NativeSnapshotCache::new("epoch-1".into());
        let snapshot = cache
            .publish(MediaState {
                source: "netease".into(),
                title: "Song A".into(),
                artist: "Artist".into(),
                ..MediaState::default()
            })
            .unwrap();
        let mut newer_poll = snapshot.clone();
        newer_poll.native_revision += 3;
        assert!(progress_read_basis_is_current(
            &newer_poll,
            "epoch-1",
            "Song A",
            "Artist",
            snapshot.native_track_generation,
            7,
            7,
            snapshot.native_revision,
        ));
        assert!(!progress_read_basis_is_current(
            &newer_poll,
            "epoch-1",
            "Song A",
            "Artist",
            snapshot.native_track_generation,
            7,
            8,
            snapshot.native_revision,
        ));
        assert!(!progress_read_basis_is_current(
            &newer_poll,
            "epoch-1",
            "Song B",
            "Artist",
            snapshot.native_track_generation,
            7,
            7,
            snapshot.native_revision,
        ));
    }

    #[test]
    fn progress_read_gate_stays_busy_until_worker_permit_drops() {
        let gate = NeteaseProgressInFlight::default();
        let first = gate.begin().expect("first read should acquire gate");
        assert!(gate.begin().is_none());
        drop(first);
        assert!(gate.begin().is_some());
    }

    #[test]
    fn track_generation_changes_on_identity_change_and_not_position_updates() {
        let mut cache = NativeSnapshotCache::new("epoch-1".into());
        let first = cache
            .publish(MediaState {
                source: "netease".into(),
                title: "Song A".into(),
                artist: "Artist".into(),
                ..MediaState::default()
            })
            .unwrap();
        let progress = cache
            .publish(MediaState {
                source: "netease".into(),
                title: "Song A".into(),
                artist: "Artist".into(),
                position_ms: 10_000,
                ..MediaState::default()
            })
            .unwrap();
        let changed = cache
            .publish(MediaState {
                source: "netease".into(),
                title: "Song B".into(),
                artist: "Artist".into(),
                ..MediaState::default()
            })
            .unwrap();
        assert_eq!(
            first.native_track_generation,
            progress.native_track_generation
        );
        assert!(changed.native_track_generation > progress.native_track_generation);
    }

    #[test]
    fn progress_scheduler_enforces_playing_paused_and_failure_intervals() {
        let now = std::time::Instant::now();
        let mut scheduler = NeteaseProgressScheduler::default();
        assert!(!scheduler.due(now, false, true));
        assert!(scheduler.due(now, true, true));
        scheduler.success(now);
        assert!(!scheduler.due(now + Duration::from_millis(199), true, true));
        assert!(scheduler.due(now + Duration::from_millis(200), true, true));
        assert!(!scheduler.due(now + Duration::from_millis(499), true, false));
        assert!(scheduler.due(now + Duration::from_millis(500), true, false));

        scheduler.failure(now + Duration::from_secs(5));
        assert!(!scheduler.due(now + Duration::from_millis(5249), true, true));
        assert!(scheduler.due(now + Duration::from_millis(5250), true, true));
        assert!(!scheduler.due(now + Duration::from_millis(5250), false, true));
    }

    #[test]
    fn progress_deadline_wakes_before_the_next_smtc_poll_without_busy_retry() {
        let now = std::time::Instant::now();
        let mut scheduler = NeteaseProgressScheduler::default();
        scheduler.success(now);
        let smtc_poll = now + Duration::from_secs(1);
        let playing_read = scheduler.next_due(now, true, true).unwrap();
        assert_eq!(
            smtc_poll.min(playing_read),
            now + Duration::from_millis(200)
        );
        assert_eq!(
            scheduler.next_due(now, true, false),
            Some(now + Duration::from_millis(500))
        );
        assert_eq!(scheduler.next_due(now, false, true), None);
        scheduler.failure(now);
        assert_eq!(
            scheduler.next_due(now, true, true),
            Some(now + Duration::from_millis(250))
        );
        scheduler.reset();
        assert!(scheduler.due(now, true, true));
        scheduler.retry_soon(now);
        assert!(!scheduler.due(now + Duration::from_millis(99), true, true));
        assert!(scheduler.due(now + Duration::from_millis(100), true, true));
    }

    #[test]
    fn progress_scheduler_reenables_with_one_immediate_real_read_after_estimate_mode() {
        let now = std::time::Instant::now();
        let mut scheduler = NeteaseProgressScheduler::default();
        scheduler.success(now);
        let estimate_selected = scheduler.mode_enabled(NeteaseProgressMode::Estimate);
        assert!(!estimate_selected);
        assert!(!scheduler.due(now + Duration::from_secs(60), estimate_selected, true));
        assert!(scheduler.mode_enabled(NeteaseProgressMode::Real));
        scheduler.reset();
        assert!(scheduler.due(now, true, true));
    }

    #[test]
    fn transient_progress_failures_retry_quickly_and_persistent_failure_stays_bounded() {
        let now = std::time::Instant::now();
        let mut scheduler = NeteaseProgressScheduler::default();
        for delay in [250, 500, 1000, 2000, 4000, 5000, 5000] {
            scheduler.failure(now);
            assert_eq!(
                scheduler.next_due(now, true, true),
                Some(now + Duration::from_millis(delay))
            );
            assert!(!scheduler.due(now, true, true));
        }
        scheduler.success(now);
        scheduler.failure(now + Duration::from_secs(1));
        assert_eq!(
            scheduler.next_due(now + Duration::from_secs(1), true, true),
            Some(now + Duration::from_millis(1250))
        );
        scheduler.reset();
        assert!(scheduler.due(now, true, true));
        assert_eq!(scheduler.next_due(now, false, true), None);
    }

    #[test]
    fn persisted_latest_mode_survives_a_full_wake_queue_and_invalidates_old_read() {
        use crate::state::{AppState, MediaActorHandle};

        let app_state = AppState::default();
        assert_eq!(app_state.netease_progress_mode(), NeteaseProgressMode::Real);
        let (sender, receiver) = mpsc::sync_channel(1);
        assert!(sender.try_send(MediaActorMessage::Refresh).is_ok());
        let actor = MediaActorHandle::new(sender);

        // Both wake markers are dropped because the queue is full, but the
        // shared persisted value retains the latest choice.
        app_state.set_netease_progress_mode(NeteaseProgressMode::Estimate);
        assert_eq!(
            app_state.netease_progress_mode(),
            NeteaseProgressMode::Estimate
        );
        actor.wake_for_netease_progress_mode_change();
        app_state.set_netease_progress_mode(NeteaseProgressMode::Real);
        actor.wake_for_netease_progress_mode_change();
        assert_eq!(app_state.netease_progress_mode(), NeteaseProgressMode::Real);

        let mut current = NeteaseProgressMode::Estimate;
        let mut generation = 8;
        let mut scheduler = NeteaseProgressScheduler::default();
        scheduler.failure(std::time::Instant::now());
        let inflight = NeteaseProgressInFlight::default();
        let permit = inflight.begin().expect("old worker owns the gate");
        assert!(reconcile_netease_progress_mode(
            &mut current,
            app_state.netease_progress_mode(),
            &mut generation,
            &mut scheduler,
        ));
        assert_eq!(current, NeteaseProgressMode::Real);
        assert_eq!(generation, 9);
        assert!(scheduler.due(std::time::Instant::now(), true, true));
        assert!(
            inflight.begin().is_none(),
            "mode change must not release the old worker gate"
        );
        drop(permit);
        assert!(
            inflight.begin().is_some(),
            "the worker releases its own gate on exit"
        );
        let snapshot = NativeSnapshotCache::new("epoch-mode".into())
            .publish(MediaState {
                source: "netease".into(),
                title: "Song".into(),
                artist: "Artist".into(),
                ..MediaState::default()
            })
            .unwrap();
        assert!(!progress_read_basis_is_current(
            &snapshot,
            "epoch-mode",
            "Song",
            "Artist",
            snapshot.native_track_generation,
            8,
            generation,
            snapshot.native_revision,
        ));
        assert!(matches!(
            receiver.try_recv(),
            Ok(MediaActorMessage::Refresh)
        ));
    }

    #[test]
    fn real_progress_anchor_survives_empty_smtc_and_accepts_external_seek_refresh() {
        let now = std::time::Instant::now();
        let mut cache = NativeSnapshotCache::new("epoch-1".into());
        let first = cache
            .publish(MediaState {
                source: "netease".into(),
                title: "Song".into(),
                artist: "Artist".into(),
                is_playing: true,
                duration_ms: 180_000,
                ..MediaState::default()
            })
            .unwrap();
        let first_progress = NeteaseProgress {
            track_id: "track-1".into(),
            play_id: "play-1".into(),
            title: "Song".into(),
            artist: "Artist".into(),
            position_ms: 30_000,
            duration_ms: 180_000,
        };
        let actual = cache
            .publish_netease_progress(
                &first.native_epoch,
                &first.title,
                &first.artist,
                first.native_track_generation,
                &first_progress,
            )
            .unwrap();
        let mut sample = NeteaseTimelineSample::from_progress(&actual, &first_progress, now);
        assert_eq!(sample.cdp_identity(), ("track-1", "play-1"));

        let mut empty_smtc = MediaState {
            source: "netease".into(),
            title: "Song".into(),
            artist: "Artist".into(),
            is_playing: true,
            position_ms: 0,
            duration_ms: 0,
            native_epoch: actual.native_epoch.clone(),
            native_track_generation: actual.native_track_generation,
            ..MediaState::default()
        };
        assert!(sample.apply(&mut empty_smtc, now + Duration::from_secs(9)));
        assert_eq!(empty_smtc.position_ms, 30_000);
        assert_eq!(empty_smtc.duration_ms, 180_000);

        let external_seek = NeteaseProgress {
            track_id: "track-1".into(),
            play_id: "play-1".into(),
            title: "Song".into(),
            artist: "Artist".into(),
            position_ms: 91_000,
            duration_ms: 180_000,
        };
        let refreshed = cache
            .publish_netease_progress(
                &actual.native_epoch,
                &actual.title,
                &actual.artist,
                actual.native_track_generation,
                &external_seek,
            )
            .unwrap();
        sample = NeteaseTimelineSample::from_progress(
            &refreshed,
            &external_seek,
            now + Duration::from_secs(10),
        );
        assert!(sample.apply(&mut empty_smtc, now + Duration::from_secs(10)));
        assert_eq!(empty_smtc.position_ms, 91_000);
    }

    #[test]
    fn real_progress_anchor_rebases_pause_and_resume_without_rewinding_play_state() {
        let now = std::time::Instant::now();
        let snapshot = MediaState {
            source: "netease".into(),
            title: "Song".into(),
            artist: "Artist".into(),
            is_playing: true,
            native_epoch: "epoch-1".into(),
            native_track_generation: 3,
            ..MediaState::default()
        };
        let progress = NeteaseProgress {
            track_id: "track-1".into(),
            play_id: "play-1".into(),
            title: "Song".into(),
            artist: "Artist".into(),
            position_ms: 20_000,
            duration_ms: 180_000,
        };
        let mut sample = NeteaseTimelineSample::from_progress(&snapshot, &progress, now);
        let mut current = snapshot.clone();
        current.is_playing = false;
        assert!(sample.apply(&mut current, now + Duration::from_secs(3)));
        assert_eq!(current.position_ms, 23_000);
        assert!(!current.is_playing);
        current.is_playing = true;
        assert!(sample.apply(&mut current, now + Duration::from_secs(9)));
        assert_eq!(current.position_ms, 23_000);
        assert!(current.is_playing);
    }

    #[test]
    fn verified_anchor_only_projects_a_short_seek_confirmation_transition() {
        let now = std::time::Instant::now();
        let mut anchor = NeteaseSeekAnchor {
            title: "Song".into(),
            artist: "Artist".into(),
            position_ms: 40_000,
            duration_ms: 180_000,
            is_playing: true,
            started_at: now,
            updated_at: now,
        };
        let mut snapshot = MediaState {
            source: "netease".into(),
            title: "Song".into(),
            artist: "Artist".into(),
            is_playing: true,
            position_ms: 0,
            duration_ms: 0,
            ..MediaState::default()
        };
        assert!(anchor.apply(&mut snapshot, now + Duration::from_secs(3), false));
        assert_eq!(snapshot.position_ms, 43_000);
        assert_eq!(snapshot.duration_ms, 180_000);

        snapshot.is_playing = false;
        snapshot.position_ms = 0;
        assert!(anchor.apply(&mut snapshot, now + Duration::from_secs(5), false));
        assert_eq!(snapshot.position_ms, 45_000);
        snapshot.is_playing = true;
        assert!(!anchor.apply(&mut snapshot, now + Duration::from_secs(9), false));
        assert_eq!(snapshot.position_ms, 45_000);
        assert!(!anchor.apply(
            &mut MediaState {
                title: "Other".into(),
                ..snapshot.clone()
            },
            now + Duration::from_secs(10),
            false,
        ));
        assert!(!anchor.apply(
            &mut MediaState {
                source: "netease".into(),
                title: "Song".into(),
                artist: "Artist".into(),
                ..MediaState::default()
            },
            now + Duration::from_secs(15),
            false,
        ));
    }

    #[test]
    fn estimate_mode_keeps_last_verified_seek_anchor_beyond_real_mode_expiry() {
        let now = std::time::Instant::now();
        let mut anchor = NeteaseSeekAnchor {
            title: "Song".into(),
            artist: "Artist".into(),
            position_ms: 40_000,
            duration_ms: 180_000,
            is_playing: true,
            started_at: now,
            updated_at: now,
        };
        let mut snapshot = MediaState {
            source: "netease".into(),
            title: "Song".into(),
            artist: "Artist".into(),
            is_playing: true,
            ..MediaState::default()
        };
        let later = now + Duration::from_secs(60);
        assert!(anchor.apply(&mut snapshot, later, true));
        assert_eq!(snapshot.position_ms, 100_000);
        assert!(snapshot.progress_known);
    }

    #[tokio::test]
    async fn actor_handle_reports_unavailable_and_bounded_queue_backpressure() {
        let (sender, receiver) = mpsc::sync_channel(1);
        sender.try_send(MediaActorMessage::Refresh).unwrap();
        let handle = MediaActorHandle::new(sender.clone());
        assert!(handle
            .try_request(MediaActorMessage::Refresh)
            .unwrap_err()
            .contains("queue is full"));
        drop(receiver);
        let error = match handle.snapshot().await {
            Ok(_) => panic!("disconnected actor must not return a snapshot"),
            Err(error) => error,
        };
        assert!(error.contains("unavailable"));
    }

    #[tokio::test]
    async fn waiting_snapshot_reply_has_a_finite_timeout() {
        let (_reply, response) = mpsc::sync_channel::<Result<(), String>>(1);
        let error = await_response_timeout(response, "test request", Duration::from_millis(5))
            .await
            .unwrap_err();
        assert!(error.contains("timed out"));
    }
}
