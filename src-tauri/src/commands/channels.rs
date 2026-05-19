use crate::models::Channel;
use crate::signal::manager::ManagerHandle;
use tauri::State;

#[tauri::command]
pub async fn get_channels(handle: State<'_, ManagerHandle>) -> Result<Vec<Channel>, String> {
    handle.get_channels().await
}

#[cfg(test)]
mod tests {
    use crate::models::Channel;

    #[test]
    fn channel_is_group_false_by_default() {
        let ch = Channel {
            id: "abc".into(),
            name: "Bob".into(),
            last_message: None,
            last_message_time: None,
            unread_count: 0,
            is_group: false,
        };
        assert!(!ch.is_group);
    }
}
