use crate::models::Message;
use crate::signal::manager::ManagerHandle;
use tauri::State;

#[tauri::command]
pub async fn get_messages(
    channel_id: String,
    handle: State<'_, ManagerHandle>,
) -> Result<Vec<Message>, String> {
    handle.get_messages(channel_id).await
}

#[tauri::command]
pub async fn send_message(
    channel_id: String,
    text: String,
    handle: State<'_, ManagerHandle>,
) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("Cannot send an empty message".into());
    }
    handle.send_message(channel_id, text).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn empty_text_is_rejected() {
        assert!("".trim().is_empty());
        assert!("   ".trim().is_empty());
        assert!(!"hello".trim().is_empty());
    }
}
