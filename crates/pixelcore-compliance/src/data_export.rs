use crate::models::{DataExportRequest, ExportFormat};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("Export request not found")]
    RequestNotFound,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("IO error: {0}")]
    IoError(String),
}

pub type ExportResult<T> = Result<T, ExportError>;

/// 用户数据（用于导出）
#[derive(Debug, Clone, Serialize)]
pub struct UserData {
    pub user_id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: String,
    pub profile: HashMap<String, serde_json::Value>,
    pub consents: Vec<serde_json::Value>,
    pub activities: Vec<serde_json::Value>,
}

/// 数据导出管理器
#[derive(Debug, Clone)]
pub struct DataExporter {
    requests: Arc<Mutex<HashMap<Uuid, DataExportRequest>>>,
}

impl DataExporter {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 创建导出请求
    pub fn create_export_request(
        &self,
        user_id: Uuid,
        format: ExportFormat,
    ) -> ExportResult<DataExportRequest> {
        let request = DataExportRequest::new(user_id, format);
        let mut requests = self.requests.lock().unwrap();
        requests.insert(request.id, request.clone());
        Ok(request)
    }

    /// 获取导出请求
    pub fn get_export_request(&self, request_id: Uuid) -> ExportResult<DataExportRequest> {
        let requests = self.requests.lock().unwrap();
        requests
            .get(&request_id)
            .cloned()
            .ok_or(ExportError::RequestNotFound)
    }

    /// 导出用户数据为 JSON
    pub fn export_to_json(&self, user_data: &UserData) -> ExportResult<String> {
        serde_json::to_string_pretty(user_data)
            .map_err(|e| ExportError::SerializationError(e.to_string()))
    }

    /// 导出用户数据为 CSV
    pub fn export_to_csv(&self, user_data: &UserData) -> ExportResult<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        // 写入基本信息
        wtr.write_record(&["Field", "Value"])
            .map_err(|e| ExportError::SerializationError(e.to_string()))?;
        wtr.write_record(&["User ID", &user_data.user_id.to_string()])
            .map_err(|e| ExportError::SerializationError(e.to_string()))?;
        wtr.write_record(&["Email", &user_data.email])
            .map_err(|e| ExportError::SerializationError(e.to_string()))?;
        wtr.write_record(&["Name", &user_data.name])
            .map_err(|e| ExportError::SerializationError(e.to_string()))?;
        wtr.write_record(&["Created At", &user_data.created_at])
            .map_err(|e| ExportError::SerializationError(e.to_string()))?;

        // 写入 profile 数据
        wtr.write_record(&["", ""])
            .map_err(|e| ExportError::SerializationError(e.to_string()))?;
        wtr.write_record(&["Profile Data", ""])
            .map_err(|e| ExportError::SerializationError(e.to_string()))?;
        for (key, value) in &user_data.profile {
            wtr.write_record(&[key, &value.to_string()])
                .map_err(|e| ExportError::SerializationError(e.to_string()))?;
        }

        let data = wtr
            .into_inner()
            .map_err(|e| ExportError::SerializationError(e.to_string()))?;
        String::from_utf8(data).map_err(|e| ExportError::SerializationError(e.to_string()))
    }

    /// 执行导出
    pub fn execute_export(
        &self,
        request_id: Uuid,
        user_data: &UserData,
    ) -> ExportResult<String> {
        let mut requests = self.requests.lock().unwrap();
        let request = requests
            .get_mut(&request_id)
            .ok_or(ExportError::RequestNotFound)?;

        let exported_data = match request.format {
            ExportFormat::Json => self.export_to_json(user_data)?,
            ExportFormat::Csv => self.export_to_csv(user_data)?,
        };

        request.completed_at = Some(chrono::Utc::now());

        Ok(exported_data)
    }

    /// 获取用户的所有导出请求
    pub fn get_user_export_requests(&self, user_id: Uuid) -> Vec<DataExportRequest> {
        let requests = self.requests.lock().unwrap();
        requests
            .values()
            .filter(|r| r.user_id == user_id)
            .cloned()
            .collect()
    }
}

impl Default for DataExporter {
    fn default() -> Self {
        Self::new()
    }
}
