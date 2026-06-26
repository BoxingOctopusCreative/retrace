use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::tracer::ImageTracer;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BackendId {
    #[serde(rename = "vtracer")]
    Vtracer,
    #[serde(rename = "live")]
    Live,
    #[serde(rename = "starvector-1b")]
    StarVector1B,
    #[serde(rename = "starvector-8b")]
    StarVector8B,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum BackendState {
    Ready,
    NotInstalled,
    Incompatible(String),
    Installing(f32),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStatus {
    pub id: BackendId,
    pub state: BackendState,
}

pub struct AppState {
    pub tracer: Mutex<Box<dyn ImageTracer>>,
    pub backend_statuses: Mutex<Vec<BackendStatus>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_id_serializes_to_expected_strings() {
        assert_eq!(serde_json::to_string(&BackendId::Vtracer).unwrap(), r#""vtracer""#);
        assert_eq!(serde_json::to_string(&BackendId::Live).unwrap(), r#""live""#);
        assert_eq!(serde_json::to_string(&BackendId::StarVector1B).unwrap(), r#""starvector-1b""#);
        assert_eq!(serde_json::to_string(&BackendId::StarVector8B).unwrap(), r#""starvector-8b""#);
    }

    #[test]
    fn backend_id_deserializes_from_strings() {
        assert_eq!(
            serde_json::from_str::<BackendId>(r#""vtracer""#).unwrap(),
            BackendId::Vtracer
        );
        assert_eq!(
            serde_json::from_str::<BackendId>(r#""starvector-1b""#).unwrap(),
            BackendId::StarVector1B
        );
    }

    #[test]
    fn backend_state_ready_tag_only() {
        let json = serde_json::to_string(&BackendState::Ready).unwrap();
        assert_eq!(json, r#"{"type":"ready"}"#);
    }

    #[test]
    fn backend_state_not_installed_tag_only() {
        let json = serde_json::to_string(&BackendState::NotInstalled).unwrap();
        assert_eq!(json, r#"{"type":"not_installed"}"#);
    }

    #[test]
    fn backend_state_installing_with_value() {
        let json = serde_json::to_string(&BackendState::Installing(0.5)).unwrap();
        assert_eq!(json, r#"{"type":"installing","value":0.5}"#);
    }

    #[test]
    fn backend_state_error_with_message() {
        let json = serde_json::to_string(&BackendState::Error("disk full".to_string())).unwrap();
        assert_eq!(json, r#"{"type":"error","value":"disk full"}"#);
    }

    #[test]
    fn backend_state_incompatible_with_reason() {
        let json = serde_json::to_string(&BackendState::Incompatible("no CUDA".to_string())).unwrap();
        assert_eq!(json, r#"{"type":"incompatible","value":"no CUDA"}"#);
    }

    #[test]
    fn backend_status_round_trips_through_json() {
        let original = BackendStatus {
            id: BackendId::StarVector8B,
            state: BackendState::Installing(0.75),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: BackendStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, BackendId::StarVector8B);
        assert!(matches!(parsed.state, BackendState::Installing(v) if (v - 0.75).abs() < 1e-6));
    }
}
