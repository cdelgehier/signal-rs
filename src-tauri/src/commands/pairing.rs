use crate::models::PairingStatus;
use crate::signal::manager::ManagerHandle;
use tauri::State;

#[tauri::command]
pub async fn generate_qr_code(handle: State<'_, ManagerHandle>) -> Result<String, String> {
    handle.generate_qr().await
}

#[tauri::command]
pub async fn get_pairing_status(handle: State<'_, ManagerHandle>) -> Result<PairingStatus, String> {
    handle.get_pairing_status().await
}

#[cfg(test)]
mod tests {
    use crate::models::PairingStatus;

    #[test]
    fn pairing_status_serializes() {
        assert_eq!(
            serde_json::to_string(&PairingStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&PairingStatus::Linked).unwrap(),
            "\"linked\""
        );
    }
}
