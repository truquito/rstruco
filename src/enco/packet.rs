use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct Packet {
  pub destination: Vec<String>,
  pub message: String,
}