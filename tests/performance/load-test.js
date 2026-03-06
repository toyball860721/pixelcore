import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const apiLatency = new Trend('api_latency');

// Test configuration
export const options = {
  stages: [
    { duration: '2m', target: 100 },   // Ramp up to 100 users
    { duration: '5m', target: 100 },   // Stay at 100 users
    { duration: '2m', target: 200 },   // Ramp up to 200 users
    { duration: '5m', target: 200 },   // Stay at 200 users
    { duration: '2m', target: 500 },   // Ramp up to 500 users
    { duration: '5m', target: 500 },   // Stay at 500 users
    { duration: '2m', target: 0 },     // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<100', 'p(99)<200'],  // 95% < 100ms, 99% < 200ms
    http_req_failed: ['rate<0.01'],                  // Error rate < 1%
    errors: ['rate<0.01'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  // Test API endpoint
  const apiRes = http.get(`${BASE_URL}/api/v1/health`);

  check(apiRes, {
    'API status is 200': (r) => r.status === 200,
    'API response time < 100ms': (r) => r.timings.duration < 100,
  });

  errorRate.add(apiRes.status !== 200);
  apiLatency.add(apiRes.timings.duration);

  // Test search endpoint
  const searchRes = http.get(`${BASE_URL}/api/v1/search?q=test`);

  check(searchRes, {
    'Search status is 200': (r) => r.status === 200,
    'Search response time < 50ms': (r) => r.timings.duration < 50,
  });

  errorRate.add(searchRes.status !== 200);

  // Test AI recommendation endpoint
  const aiRes = http.get(`${BASE_URL}/api/v1/recommend?user_id=123`);

  check(aiRes, {
    'AI status is 200': (r) => r.status === 200,
    'AI response time < 100ms': (r) => r.timings.duration < 100,
  });

  errorRate.add(aiRes.status !== 200);

  sleep(1);
}

export function handleSummary(data) {
  return {
    'summary.json': JSON.stringify(data),
    stdout: textSummary(data, { indent: ' ', enableColors: true }),
  };
}

function textSummary(data, options) {
  const indent = options.indent || '';
  const enableColors = options.enableColors || false;

  let summary = `
${indent}Test Summary:
${indent}=============
${indent}
${indent}Total Requests: ${data.metrics.http_reqs.values.count}
${indent}Failed Requests: ${data.metrics.http_req_failed.values.passes}
${indent}Request Rate: ${data.metrics.http_reqs.values.rate.toFixed(2)} req/s
${indent}
${indent}Response Times:
${indent}  Min: ${data.metrics.http_req_duration.values.min.toFixed(2)}ms
${indent}  Avg: ${data.metrics.http_req_duration.values.avg.toFixed(2)}ms
${indent}  Max: ${data.metrics.http_req_duration.values.max.toFixed(2)}ms
${indent}  P95: ${data.metrics.http_req_duration.values['p(95)'].toFixed(2)}ms
${indent}  P99: ${data.metrics.http_req_duration.values['p(99)'].toFixed(2)}ms
${indent}
${indent}Error Rate: ${(data.metrics.errors.values.rate * 100).toFixed(2)}%
${indent}
${indent}Virtual Users:
${indent}  Min: ${data.metrics.vus.values.min}
${indent}  Max: ${data.metrics.vus.values.max}
  `;

  return summary;
}
