use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct PoloResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct BlockResponse {
    pub block_number: String,
    pub block_number_decimal: u64,
}

#[derive(Serialize)]
pub struct ApiEndpoint {
    pub path: String,
    pub method: String,
    pub description: String,
    pub example_request: Option<String>,
    pub example_response: String,
    pub performance: Option<String>,
}

#[derive(Serialize)]
pub struct ApiDocs {
    pub name: String,
    pub version: String,
    pub base_url: String,
    pub endpoints: Vec<ApiEndpoint>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DbRecord {
    pub id: String,
    pub data: serde_json::Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRecordRequest {
    pub data: serde_json::Value,
}

#[derive(Serialize)]
pub struct DbResponse {
    pub success: bool,
    pub record: Option<DbRecord>,
    pub message: String,
}

#[derive(Serialize)]
pub struct DbListResponse {
    pub success: bool,
    pub records: Vec<DbRecord>,
    pub count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MonitorAddressRequest {
    pub address: String,
}

#[derive(Serialize)]
pub struct MonitorAddressResponse {
    pub success: bool,
    pub message: String,
    pub monitored_addresses: Vec<String>,
}

#[derive(Serialize)]
pub struct MonitorListResponse {
    pub success: bool,
    pub addresses: Vec<String>,
    pub count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransactionAlert {
    pub block_number: u64,
    pub block_hash: String,
    pub address: String,
    pub role: String,
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: String,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub timestamp: u64,
    #[serde(default = "default_category")]
    pub category: String, // "Deployment" or "Other"
    #[serde(default)]
    pub input_data: Option<String>, // Transaction input data for classification
}

fn default_category() -> String {
    "Other".to_string()
}

#[derive(Serialize)]
pub struct TransactionAlertsResponse {
    pub success: bool,
    pub alerts: Vec<TransactionAlert>,
    pub count: usize,
}

// Helper function to deserialize timestamp from either u64 or hex/decimal string
pub fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct TimestampVisitor;

    impl<'de> Visitor<'de> for TimestampVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a u64 number or a hex string")
        }

        fn visit_u64<E>(self, value: u64) -> Result<u64, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_str<E>(self, value: &str) -> Result<u64, E>
        where
            E: de::Error,
        {
            if value.starts_with("0x") || value.starts_with("0X") {
                u64::from_str_radix(&value[2..], 16)
                    .map_err(|_| E::custom(format!("invalid hex timestamp: {}", value)))
            } else {
                value
                    .parse::<u64>()
                    .map_err(|_| E::custom(format!("invalid timestamp: {}", value)))
            }
        }
    }

    deserializer.deserialize_any(TimestampVisitor)
}


