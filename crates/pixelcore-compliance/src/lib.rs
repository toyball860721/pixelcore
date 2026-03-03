pub mod audit;
pub mod data_deletion;
pub mod data_export;
pub mod gdpr;
pub mod models;

#[cfg(test)]
mod tests;

// Re-exports
pub use audit::{
    AuditError, AuditResult, AuditStatistics, ComplianceReporter, ImmutableAuditLogger,
};
pub use data_deletion::{DataDeleter, DeletionError, DeletionResult, DeletionStatistics};
pub use data_export::{DataExporter, ExportError, ExportResult, UserData};
pub use gdpr::{GdprError, GdprManager, GdprResult, GdprStatistics};
pub use models::{
    ComplianceReport, ComplianceReportType, ConsentRecord, DataDeletionRequest,
    DataExportRequest, DataSubjectRequest, DataSubjectRight, DeletionType, ExportFormat,
    ImmutableAuditLog, RequestStatus, RetentionPolicy,
};
