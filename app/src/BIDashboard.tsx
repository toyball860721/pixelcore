import React, { useState, useEffect } from 'react';
import {
  LineChart, Line, BarChart, Bar, PieChart, Pie, AreaChart, Area,
  XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, Cell
} from 'recharts';
import { format } from 'date-fns';

interface MetricData {
  timestamp: string;
  value: number;
  label?: string;
}

interface ChartConfig {
  type: 'line' | 'bar' | 'pie' | 'area';
  title: string;
  dataKey: string;
  color: string;
}

const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042', '#8884D8', '#82CA9D'];

export const BIDashboard: React.FC = () => {
  const [selectedTimeRange, setSelectedTimeRange] = useState<'1h' | '24h' | '7d' | '30d'>('24h');
  const [refreshInterval, setRefreshInterval] = useState<number>(30000); // 30 seconds
  const [isAutoRefresh, setIsAutoRefresh] = useState<boolean>(true);

  // Sample data - in production, this would come from the analytics API
  const [metricsData, setMetricsData] = useState<{
    events: MetricData[];
    performance: MetricData[];
    users: MetricData[];
    distribution: MetricData[];
  }>({
    events: [],
    performance: [],
    users: [],
    distribution: []
  });

  // Generate sample data
  useEffect(() => {
    const generateSampleData = () => {
      const now = Date.now();
      const points = 24;
      const interval = 3600000; // 1 hour

      const events = Array.from({ length: points }, (_, i) => ({
        timestamp: format(new Date(now - (points - i) * interval), 'HH:mm'),
        value: Math.floor(Math.random() * 1000) + 500,
      }));

      const performance = Array.from({ length: points }, (_, i) => ({
        timestamp: format(new Date(now - (points - i) * interval), 'HH:mm'),
        value: Math.random() * 100 + 20,
      }));

      const users = Array.from({ length: points }, (_, i) => ({
        timestamp: format(new Date(now - (points - i) * interval), 'HH:mm'),
        value: Math.floor(Math.random() * 500) + 100,
      }));

      const distribution = [
        { label: 'Page Views', value: 4500 },
        { label: 'Clicks', value: 3200 },
        { label: 'Purchases', value: 1800 },
        { label: 'Sign Ups', value: 950 },
      ];

      setMetricsData({ events, performance, users, distribution });
    };

    generateSampleData();

    if (isAutoRefresh) {
      const interval = setInterval(generateSampleData, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [refreshInterval, isAutoRefresh, selectedTimeRange]);

  const renderLineChart = (data: MetricData[], title: string, color: string) => (
    <div className="chart-container">
      <h3>{title}</h3>
      <ResponsiveContainer width="100%" height={250}>
        <LineChart data={data}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="timestamp" />
          <YAxis />
          <Tooltip />
          <Legend />
          <Line type="monotone" dataKey="value" stroke={color} strokeWidth={2} />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );

  const renderBarChart = (data: MetricData[], title: string, color: string) => (
    <div className="chart-container">
      <h3>{title}</h3>
      <ResponsiveContainer width="100%" height={250}>
        <BarChart data={data}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="timestamp" />
          <YAxis />
          <Tooltip />
          <Legend />
          <Bar dataKey="value" fill={color} />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );

  const renderAreaChart = (data: MetricData[], title: string, color: string) => (
    <div className="chart-container">
      <h3>{title}</h3>
      <ResponsiveContainer width="100%" height={250}>
        <AreaChart data={data}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="timestamp" />
          <YAxis />
          <Tooltip />
          <Legend />
          <Area type="monotone" dataKey="value" stroke={color} fill={color} fillOpacity={0.6} />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );

  const renderPieChart = (data: MetricData[], title: string) => (
    <div className="chart-container">
      <h3>{title}</h3>
      <ResponsiveContainer width="100%" height={250}>
        <PieChart>
          <Pie
            data={data}
            dataKey="value"
            nameKey="label"
            cx="50%"
            cy="50%"
            outerRadius={80}
            label
          >
            {data.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
            ))}
          </Pie>
          <Tooltip />
          <Legend />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );

  const renderSummaryCard = (title: string, value: string | number, change: string, isPositive: boolean) => (
    <div className="summary-card">
      <h4>{title}</h4>
      <div className="summary-value">{value}</div>
      <div className={`summary-change ${isPositive ? 'positive' : 'negative'}`}>
        {isPositive ? '↑' : '↓'} {change}
      </div>
    </div>
  );

  return (
    <div className="bi-dashboard">
      <div className="dashboard-header">
        <h1>📊 BI Dashboard</h1>
        <div className="dashboard-controls">
          <div className="time-range-selector">
            <button
              className={selectedTimeRange === '1h' ? 'active' : ''}
              onClick={() => setSelectedTimeRange('1h')}
            >
              1H
            </button>
            <button
              className={selectedTimeRange === '24h' ? 'active' : ''}
              onClick={() => setSelectedTimeRange('24h')}
            >
              24H
            </button>
            <button
              className={selectedTimeRange === '7d' ? 'active' : ''}
              onClick={() => setSelectedTimeRange('7d')}
            >
              7D
            </button>
            <button
              className={selectedTimeRange === '30d' ? 'active' : ''}
              onClick={() => setSelectedTimeRange('30d')}
            >
              30D
            </button>
          </div>
          <div className="refresh-controls">
            <label>
              <input
                type="checkbox"
                checked={isAutoRefresh}
                onChange={(e) => setIsAutoRefresh(e.target.checked)}
              />
              Auto Refresh
            </label>
            <select
              value={refreshInterval}
              onChange={(e) => setRefreshInterval(Number(e.target.value))}
              disabled={!isAutoRefresh}
            >
              <option value={10000}>10s</option>
              <option value={30000}>30s</option>
              <option value={60000}>1m</option>
              <option value={300000}>5m</option>
            </select>
          </div>
        </div>
      </div>

      <div className="summary-cards">
        {renderSummaryCard('Total Events', '125.4K', '+12.5%', true)}
        {renderSummaryCard('Active Users', '8,432', '+8.2%', true)}
        {renderSummaryCard('Avg Response Time', '45ms', '-5.3%', true)}
        {renderSummaryCard('Error Rate', '0.12%', '+0.05%', false)}
      </div>

      <div className="charts-grid">
        <div className="chart-row">
          {renderLineChart(metricsData.events, 'Events Over Time', '#0088FE')}
          {renderAreaChart(metricsData.users, 'Active Users', '#00C49F')}
        </div>
        <div className="chart-row">
          {renderBarChart(metricsData.performance, 'Response Time (ms)', '#FFBB28')}
          {renderPieChart(metricsData.distribution, 'Event Distribution')}
        </div>
      </div>

      <style>{`
        .bi-dashboard {
          padding: 20px;
          background: #f5f5f5;
          min-height: 100vh;
        }

        .dashboard-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 30px;
          background: white;
          padding: 20px;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .dashboard-header h1 {
          margin: 0;
          font-size: 28px;
          color: #333;
        }

        .dashboard-controls {
          display: flex;
          gap: 20px;
          align-items: center;
        }

        .time-range-selector button {
          padding: 8px 16px;
          margin: 0 4px;
          border: 1px solid #ddd;
          background: white;
          border-radius: 4px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .time-range-selector button:hover {
          background: #f0f0f0;
        }

        .time-range-selector button.active {
          background: #0088FE;
          color: white;
          border-color: #0088FE;
        }

        .refresh-controls {
          display: flex;
          gap: 10px;
          align-items: center;
        }

        .refresh-controls select {
          padding: 6px 12px;
          border: 1px solid #ddd;
          border-radius: 4px;
        }

        .summary-cards {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
          gap: 20px;
          margin-bottom: 30px;
        }

        .summary-card {
          background: white;
          padding: 20px;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .summary-card h4 {
          margin: 0 0 10px 0;
          color: #666;
          font-size: 14px;
          font-weight: 500;
        }

        .summary-value {
          font-size: 32px;
          font-weight: bold;
          color: #333;
          margin-bottom: 8px;
        }

        .summary-change {
          font-size: 14px;
          font-weight: 500;
        }

        .summary-change.positive {
          color: #00C49F;
        }

        .summary-change.negative {
          color: #FF8042;
        }

        .charts-grid {
          display: flex;
          flex-direction: column;
          gap: 20px;
        }

        .chart-row {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
          gap: 20px;
        }

        .chart-container {
          background: white;
          padding: 20px;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .chart-container h3 {
          margin: 0 0 20px 0;
          font-size: 18px;
          color: #333;
        }

        @media (max-width: 1200px) {
          .chart-row {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};

export default BIDashboard;
