/// Implements NEP-297 (Events standard) parsing.
/// https://github.com/near/NEPs/pull/298/files
use tonic_perps_sdk::event_types::EventType as PerpsEvent;

// const EVENT_PREFIX: &'static str = "EVENT_JSON:";
// All Tonic logs are events.
const EVENT_PREFIX: &str = "";
const PREFIX_LENGTH: usize = EVENT_PREFIX.len();

/// Return true if the string starts with [`EVENT_PREFIX`]
pub fn is_event_log(s: &str) -> bool {
    // true
    s.starts_with(EVENT_PREFIX)
}

fn extract_event_str(s: &str) -> String {
    s.chars().skip(PREFIX_LENGTH).collect()
}

pub fn parse_event(s: &str) -> Result<PerpsEvent, serde_json::Error> {
    let extracted = extract_event_str(s);
    serde_json::from_str::<PerpsEvent>(&extracted)
}
