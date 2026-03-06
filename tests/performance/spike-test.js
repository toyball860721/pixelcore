import http from 'k6/http';
import { check, sleep } from 'k6';

// Spike test - sudden traffic increase
export const options = {
  stages: [
    { duration: '1m', target: 100 },    // Normal load
    { duration: '30s', target: 2000 },  // Sudden spike!
    { duration: '3m', target: 2000 },   // Stay at spike
    { duration: '1m', target: 100 },    // Back to normal
    { duration: '1m', target: 0 },      // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<200'],
    http_req_failed: ['rate<0.1'],      // Allow 10% errors during spike
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  const res = http.get(`${BASE_URL}/api/v1/health`);

  check(res, {
    'status is 200': (r) => r.status === 200,
  });

  sleep(0.1);
}
