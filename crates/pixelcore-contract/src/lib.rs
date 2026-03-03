//! Smart Contract Engine - 智能合约引擎
//!
//! 提供合约模板、条件检查、自动执行和争议解决功能

mod models;
mod template;
mod executor;
mod validator;

pub use models::*;
pub use template::*;
pub use executor::*;
pub use validator::*;

#[cfg(test)]
mod tests;
