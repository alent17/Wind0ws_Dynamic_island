//! Read-only SMTC/CDP identity probe; never seeks, restarts or changes playback.
#[path = "../src/services/apartment.rs"]
mod apartment;
#[allow(dead_code)]
#[path = "../src/services/netease_cdp.rs"]
mod netease_cdp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use windows052::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
    let _apartment = apartment::Apartment::enter()?;
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.get()?;
    let sessions = manager.GetSessions()?;
    for index in 0..sessions.Size()? {
        let session = sessions.GetAt(index)?;
        let id = session.SourceAppUserModelId()?.to_string();
        let properties = session.TryGetMediaPropertiesAsync()?.get()?;
        let title = properties.Title()?.to_string();
        let artist = properties.Artist()?.to_string();
        let progress = netease_cdp::LocalCdpClient::default().read_progress(&title, &artist);
        let like = netease_cdp::LocalCdpClient::default().song_like(&title, &artist, None);
        println!(
            "{}",
            serde_json::json!({
                "sessionId": id,
                "title": title,
                "artist": artist,
                "likeState": like.as_ref().ok(),
                "likeError": like.as_ref().err().map(|error| error.to_string()),
                "neteaseTrackMatches": progress.is_ok(),
                "positionMs": progress.as_ref().ok().map(|p| p.position_ms),
                "durationMs": progress.as_ref().ok().map(|p| p.duration_ms),
                "connectionError": progress.err().map(|error| error.to_string()),
            })
        );
    }
    Ok(())
}
