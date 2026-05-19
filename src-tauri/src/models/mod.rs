use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub last_message: Option<String>,
    pub last_message_time: Option<i64>,
    pub unread_count: u32,
    pub is_group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: u64,
    pub sender_id: String,
    pub sender_name: String,
    pub text: Option<String>,
    pub timestamp: i64,
    pub is_outgoing: bool,
    pub receipt: Option<ReceiptStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReceiptStatus {
    Sent,
    Delivered,
    Read,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PairingStatus {
    Pending,
    Linked,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_serializes_correctly() {
        let ch = Channel {
            id: "uuid-123".into(),
            name: "Alice".into(),
            last_message: Some("Hello".into()),
            last_message_time: Some(1_700_000_000),
            unread_count: 2,
            is_group: false,
        };
        let json = serde_json::to_string(&ch).unwrap();
        assert!(json.contains("\"id\":\"uuid-123\""));
        assert!(json.contains("\"unread_count\":2"));
    }

    #[test]
    fn message_outgoing_flag() {
        let msg = Message {
            id: 42,
            sender_id: "me".into(),
            sender_name: "Me".into(),
            text: Some("Hi".into()),
            timestamp: 1_700_000_001,
            is_outgoing: true,
            receipt: Some(ReceiptStatus::Read),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"is_outgoing\":true"));
        assert!(json.contains("\"receipt\":\"read\""));
    }

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
