use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use windows052::{ApplicationModel::Package, Foundation::{EventRegistrationToken, TypedEventHandler}, UI::Notifications::{KnownNotificationBindings, UserNotification, UserNotificationChangedEventArgs, UserNotificationChangedKind, Management::{UserNotificationListener, UserNotificationListenerAccessStatus}}};
static LISTENER: Mutex<Option<(UserNotificationListener, EventRegistrationToken)>> = Mutex::new(None);
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NotificationPayload { id: u32, app: String, title: String, body: String }
fn status() -> &'static str {
    if Package::Current().is_err() { return "unavailable"; }
    match UserNotificationListener::Current().and_then(|l| l.GetAccessStatus()) {
        Ok(UserNotificationListenerAccessStatus::Allowed) => "allowed",
        Ok(UserNotificationListenerAccessStatus::Denied) => "denied",
        Ok(_) => "pending", Err(_) => "unavailable",
    }
}
#[tauri::command]
pub fn notification_status() -> &'static str { status() }
#[tauri::command]
pub async fn request_notification_access(app: AppHandle) -> Result<String, String> {
    if Package::Current().is_err() { return Ok("unavailable".into()); }
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.run_on_main_thread(move || { let _ = tx.send(UserNotificationListener::Current().and_then(|l| l.RequestAccessAsync())); }).map_err(|_| "unavailable".to_string())?;
    let op = rx.await.map_err(|_| "unavailable".to_string())?.map_err(|_| "unavailable".to_string())?;
    let result = tauri::async_runtime::spawn_blocking(move || op.get()).await.map_err(|_| "unavailable".to_string())?.map_err(|_| "unavailable".to_string())?;
    Ok(if result == UserNotificationListenerAccessStatus::Allowed { "allowed" } else if result == UserNotificationListenerAccessStatus::Denied { "denied" } else { "pending" }.into())
}
fn payload(n: &UserNotification) -> windows052::core::Result<NotificationPayload> {
    let id = n.Id()?;
    let app = n.AppInfo().and_then(|i| i.DisplayInfo()).and_then(|i| i.DisplayName()).map(|s| s.to_string()).unwrap_or_default();
    let mut lines = Vec::new();
    if let Ok(binding) = n.Notification().and_then(|v| v.Visual()).and_then(|v| v.GetBinding(&KnownNotificationBindings::ToastGeneric()?)) {
        if let Ok(elements) = binding.GetTextElements() { for item in &elements { if let Ok(text) = item.Text() { lines.push(text.to_string()); } } }
    }
    Ok(NotificationPayload { id, app, title: lines.first().cloned().unwrap_or_default(), body: lines.iter().skip(1).cloned().collect::<Vec<_>>().join(" ") })
}
#[tauri::command]
pub fn start_notification_listener(app: AppHandle) -> Result<(), String> {
    if status() != "allowed" { return Err(status().into()); }
    let mut guard = LISTENER.lock().map_err(|_| "unavailable")?;
    if guard.is_some() { return Ok(()); }
    let listener = UserNotificationListener::Current().map_err(|_| "unavailable")?;
    let token = listener.NotificationChanged(&TypedEventHandler::new(move |sender: &Option<UserNotificationListener>, args: &Option<UserNotificationChangedEventArgs>| {
        if let (Some(sender), Some(args)) = (sender, args) {
            if sender.GetAccessStatus().ok() == Some(UserNotificationListenerAccessStatus::Allowed) && args.ChangeKind().ok() == Some(UserNotificationChangedKind::Added) {
                if let Ok(id) = args.UserNotificationId() { if let Ok(data) = sender.GetNotification(id).and_then(|n| payload(&n)) { let _ = app.emit("windows-notification", data); } }
            }
        }
        Ok(())
    })).map_err(|_| "unavailable")?;
    *guard = Some((listener, token)); Ok(())
}
#[tauri::command]
pub fn stop_notification_listener() {
    if let Ok(mut guard) = LISTENER.lock() { if let Some((listener, token)) = guard.take() { let _ = listener.RemoveNotificationChanged(token); } }
}
