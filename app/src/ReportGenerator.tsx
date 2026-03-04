import React, { useState } from 'react';

interface ReportConfig {
  name: string;
  type: 'summary' | 'detailed' | 'custom';
  metrics: string[];
  timeRange: string;
  format: 'csv' | 'excel' | 'pdf';
  schedule?: string;
}

export const ReportGenerator: React.FC = () => {
  const [reportConfig, setReportConfig] = useState<ReportConfig>({
    name: '',
    type: 'summary',
    metrics: [],
    timeRange: '24h',
    format: 'csv',
  });

  const [savedReports, setSavedReports] = useState<ReportConfig[]>([]);
  const [isGenerating, setIsGenerating] = useState(false);

  const availableMetrics = [
    'Total Events',
    'Active Users',
    'Page Views',
    'Response Time',
    'Error Rate',
    'Conversion Rate',
    'Revenue',
    'User Engagement',
  ];

  const handleMetricToggle = (metric: string) => {
    setReportConfig(prev => ({
      ...prev,
      metrics: prev.metrics.includes(metric)
        ? prev.metrics.filter(m => m !== metric)
        : [...prev.metrics, metric]
    }));
  };

  const handleGenerateReport = async () => {
    if (!reportConfig.name || reportConfig.metrics.length === 0) {
      alert('Please provide a report name and select at least one metric');
      return;
    }

    setIsGenerating(true);

    // Simulate report generation
    await new Promise(resolve => setTimeout(resolve, 2000));

    // In production, this would call the backend API
    console.log('Generating report:', reportConfig);

    // Simulate download
    const blob = new Blob(['Report data...'], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${reportConfig.name}.${reportConfig.format}`;
    a.click();
    URL.revokeObjectURL(url);

    setIsGenerating(false);
    alert('Report generated successfully!');
  };

  const handleSaveReport = () => {
    if (!reportConfig.name) {
      alert('Please provide a report name');
      return;
    }

    setSavedReports(prev => [...prev, { ...reportConfig }]);
    alert('Report configuration saved!');
  };

  const handleLoadReport = (report: ReportConfig) => {
    setReportConfig(report);
  };

  return (
    <div className="report-generator">
      <h2>📄 Report Generator</h2>

      <div className="report-form">
        <div className="form-section">
          <h3>Basic Information</h3>
          <div className="form-group">
            <label>Report Name</label>
            <input
              type="text"
              value={reportConfig.name}
              onChange={(e) => setReportConfig(prev => ({ ...prev, name: e.target.value }))}
              placeholder="Enter report name"
            />
          </div>

          <div className="form-group">
            <label>Report Type</label>
            <select
              value={reportConfig.type}
              onChange={(e) => setReportConfig(prev => ({ ...prev, type: e.target.value as any }))}
            >
              <option value="summary">Summary Report</option>
              <option value="detailed">Detailed Report</option>
              <option value="custom">Custom Report</option>
            </select>
          </div>
        </div>

        <div className="form-section">
          <h3>Metrics Selection</h3>
          <div className="metrics-grid">
            {availableMetrics.map(metric => (
              <label key={metric} className="metric-checkbox">
                <input
                  type="checkbox"
                  checked={reportConfig.metrics.includes(metric)}
                  onChange={() => handleMetricToggle(metric)}
                />
                {metric}
              </label>
            ))}
          </div>
        </div>

        <div className="form-section">
          <h3>Time Range & Format</h3>
          <div className="form-row">
            <div className="form-group">
              <label>Time Range</label>
              <select
                value={reportConfig.timeRange}
                onChange={(e) => setReportConfig(prev => ({ ...prev, timeRange: e.target.value }))}
              >
                <option value="1h">Last Hour</option>
                <option value="24h">Last 24 Hours</option>
                <option value="7d">Last 7 Days</option>
                <option value="30d">Last 30 Days</option>
                <option value="custom">Custom Range</option>
              </select>
            </div>

            <div className="form-group">
              <label>Export Format</label>
              <select
                value={reportConfig.format}
                onChange={(e) => setReportConfig(prev => ({ ...prev, format: e.target.value as any }))}
              >
                <option value="csv">CSV</option>
                <option value="excel">Excel</option>
                <option value="pdf">PDF</option>
              </select>
            </div>
          </div>
        </div>

        <div className="form-section">
          <h3>Schedule (Optional)</h3>
          <div className="form-group">
            <label>Schedule Expression (Cron)</label>
            <input
              type="text"
              value={reportConfig.schedule || ''}
              onChange={(e) => setReportConfig(prev => ({ ...prev, schedule: e.target.value }))}
              placeholder="e.g., 0 9 * * * (Daily at 9 AM)"
            />
            <small>Leave empty for one-time generation</small>
          </div>
        </div>

        <div className="form-actions">
          <button
            className="btn btn-primary"
            onClick={handleGenerateReport}
            disabled={isGenerating}
          >
            {isGenerating ? 'Generating...' : 'Generate Report'}
          </button>
          <button
            className="btn btn-secondary"
            onClick={handleSaveReport}
          >
            Save Configuration
          </button>
        </div>
      </div>

      {savedReports.length > 0 && (
        <div className="saved-reports">
          <h3>Saved Report Configurations</h3>
          <div className="reports-list">
            {savedReports.map((report, index) => (
              <div key={index} className="report-item">
                <div className="report-info">
                  <h4>{report.name}</h4>
                  <p>{report.type} • {report.metrics.length} metrics • {report.format.toUpperCase()}</p>
                </div>
                <button
                  className="btn btn-small"
                  onClick={() => handleLoadReport(report)}
                >
                  Load
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      <style>{`
        .report-generator {
          padding: 20px;
          max-width: 1200px;
          margin: 0 auto;
        }

        .report-generator h2 {
          margin-bottom: 30px;
          color: #333;
        }

        .report-form {
          background: white;
          padding: 30px;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .form-section {
          margin-bottom: 30px;
        }

        .form-section h3 {
          margin-bottom: 15px;
          color: #555;
          font-size: 18px;
        }

        .form-group {
          margin-bottom: 15px;
        }

        .form-group label {
          display: block;
          margin-bottom: 5px;
          color: #666;
          font-weight: 500;
        }

        .form-group input,
        .form-group select {
          width: 100%;
          padding: 10px;
          border: 1px solid #ddd;
          border-radius: 4px;
          font-size: 14px;
        }

        .form-group small {
          display: block;
          margin-top: 5px;
          color: #999;
          font-size: 12px;
        }

        .form-row {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 20px;
        }

        .metrics-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
          gap: 10px;
        }

        .metric-checkbox {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 10px;
          border: 1px solid #ddd;
          border-radius: 4px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .metric-checkbox:hover {
          background: #f5f5f5;
        }

        .metric-checkbox input {
          width: auto;
        }

        .form-actions {
          display: flex;
          gap: 10px;
          margin-top: 30px;
        }

        .btn {
          padding: 12px 24px;
          border: none;
          border-radius: 4px;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .btn-primary {
          background: #0088FE;
          color: white;
        }

        .btn-primary:hover:not(:disabled) {
          background: #0066CC;
        }

        .btn-primary:disabled {
          background: #ccc;
          cursor: not-allowed;
        }

        .btn-secondary {
          background: #f0f0f0;
          color: #333;
        }

        .btn-secondary:hover {
          background: #e0e0e0;
        }

        .btn-small {
          padding: 6px 12px;
          font-size: 12px;
        }

        .saved-reports {
          margin-top: 40px;
        }

        .saved-reports h3 {
          margin-bottom: 15px;
          color: #333;
        }

        .reports-list {
          display: grid;
          gap: 10px;
        }

        .report-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 15px;
          background: white;
          border: 1px solid #ddd;
          border-radius: 4px;
        }

        .report-info h4 {
          margin: 0 0 5px 0;
          color: #333;
        }

        .report-info p {
          margin: 0;
          color: #666;
          font-size: 14px;
        }
      `}</style>
    </div>
  );
};

export default ReportGenerator;
