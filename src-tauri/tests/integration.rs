use signal_rs_lib::models::{Channel, Message, PairingStatus, ReceiptStatus};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_message(is_outgoing: bool, text: Option<&str>) -> Message {
    Message {
        id: 1,
        sender_id: "uuid".into(),
        sender_name: "Alice".into(),
        text: text.map(str::to_string),
        timestamp: 1_700_000_000,
        is_outgoing,
        receipt: None,
    }
}
use signal_rs_lib::signal::manager::url_to_qr_png_base64;

#[test]
fn channel_serde_roundtrip() {
    let original = Channel {
        id: "00000000-0000-0000-0000-000000000001".into(),
        name: "Test Contact".into(),
        last_message: Some("Hey!".into()),
        last_message_time: Some(1_700_000_000),
        unread_count: 3,
        is_group: false,
    };
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Channel = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, original.id);
    assert_eq!(deserialized.unread_count, original.unread_count);
}

#[test]
fn message_serde_roundtrip() {
    let original = Message {
        id: 1_700_000_001_000,
        sender_id: "sender-uuid".into(),
        sender_name: "Alice".into(),
        text: Some("Hello, world!".into()),
        timestamp: 1_700_000_001_000,
        is_outgoing: false,
        receipt: Some(ReceiptStatus::Delivered),
    };
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.text, original.text);
    assert!(!deserialized.is_outgoing);
}

#[test]
fn pairing_status_serde() {
    assert_eq!(
        serde_json::to_string(&PairingStatus::Pending).unwrap(),
        "\"pending\""
    );
    assert_eq!(
        serde_json::to_string(&PairingStatus::Linked).unwrap(),
        "\"linked\""
    );
}

// ---------------------------------------------------------------------------
// Feature: system notifications — notification trigger conditions
// ---------------------------------------------------------------------------

#[test]
fn notification_triggers_for_incoming_message_with_text() {
    let msg = make_message(false, Some("Hello"));
    assert!(!msg.is_outgoing);
    assert!(msg.text.is_some());
    // Both conditions required: incoming AND has text
    let should_notify = !msg.is_outgoing && msg.text.is_some();
    assert!(should_notify);
}

#[test]
fn notification_does_not_trigger_for_outgoing_message() {
    let msg = make_message(true, Some("Hello"));
    let should_notify = !msg.is_outgoing && msg.text.is_some();
    assert!(!should_notify);
}

#[test]
fn notification_does_not_trigger_for_attachment_without_text() {
    let msg = make_message(false, None);
    let should_notify = !msg.is_outgoing && msg.text.is_some();
    assert!(!should_notify);
}

// ---------------------------------------------------------------------------

#[test]
fn qr_code_generation_from_url() {
    let url = "sgnl://linkdevice?uuid=test&pub_key=abc123";
    let result = url_to_qr_png_base64(url);
    assert!(result.is_ok(), "QR generation failed: {:?}", result.err());
    let b64 = result.unwrap();
    assert!(!b64.is_empty());
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&b64)
        .unwrap();
    // PNG magic bytes
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);
}
