use serde::{Deserialize, Serialize};

/// An UP's profile as the space page shows it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UpProfile {
    pub mid: u64,
    pub name: String,
    pub face: String,
    pub sign: String,
    pub level: u32,
    /// 认证 text ("bilibili 知名 UP 主"…), empty when none.
    pub official: String,
    pub fans: u64,
    pub followings: u64,
    pub archives: u64,
    /// Whether the current user follows this UP (false for guests).
    pub followed: bool,
}
