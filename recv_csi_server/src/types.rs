use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Receive {
    Realtime,
    Predefined(f64, f64)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WriteConfig {
    pub write_at_least: usize,
    pub data: Receive,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CSIData {
    pub inner: Vec< // vector of samples
            Vec<Vec<Vec<f64>>>
            >,
    pub c: Option<WriteConfig>,
    pub recent_xy: (f64, f64),
    pub samples: Vec<Sample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sample {
    pub date: DateTime<Utc>,
    pub x: f64,
    pub y: f64,
    pub csi: Vec<Vec<Vec<f64>>>,
}
