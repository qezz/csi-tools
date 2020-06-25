use serde::{Deserialize, Serialize};

use crate::CSIStruct;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComplexDef<T> {
    pub re: T,
    pub im: T,
}

/// 'Absolute' value of a subcarrier
pub fn abs(c: ComplexDef<isize>) -> f64 {
    ((c.re.pow(2) +  c.im.pow(2)) as f64).sqrt()
}

/// Serialization type
#[derive(Debug, Serialize, Deserialize)]
pub struct SerCSI {
    pub csi_matrix: Vec<Vec<Vec<ComplexDef<isize>>>>,
    pub csi_status: CSIStruct,
}
