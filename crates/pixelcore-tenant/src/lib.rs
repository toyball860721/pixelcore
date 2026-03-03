//! Tenant System - 多租户系统
//!
//! 提供租户管理、资源配额和数据隔离功能

mod models;
mod manager;

pub use models::*;
pub use manager::*;

#[cfg(test)]
mod tests;
