//! Marketplace - Agent 服务发现和匹配系统
//!
//! 提供 Agent 服务的发现、搜索、匹配和推荐功能

mod discovery;
mod matcher;
mod catalog;

pub use discovery::*;
pub use matcher::*;
pub use catalog::*;

#[cfg(test)]
mod tests;
