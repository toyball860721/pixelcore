import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

// Stress test configuration - push system to its limits
export const options = {
  stages: [
    { duration: '2m', target: 500 },    // Ramp up to 500 users
    { duration: '5m', target: 500 },    // Stay at 500
    { duration: '2m', target: 1000 },   // Ramp up to 1000
    { duration: '5m', target: 1000 },   // Stay at 1000
    { duration: '2m', target: 2000 },   // Ramp up to 2000
    { duration: '5m', target: 2000 },   // Stay at 2000
    { duration: '5m', target: 3000 },   // Push to 3000
    { duration: '2m', target: 0 },      // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(99)<500'],   // 99% < 500ms under stress
    http_req_failed: ['rate<0.05'],     // Error rate < 5% under stress
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  const responses = http.batch([
    ['GET', `${BASE_URL}/api/v1/health`],
    ['GET', `${BASE_URL}/api/v1/search?q=stress`],
    ['GET', `${BASE_URL}/api/v1/recommend?user_id=456`],
  ]);

  responses.forEach((res) => {
    check(res, {
      'status is 200 or 503': (r) => r.status === 200 || r.status === 503,
    });
    errorRate.add(res.status !== 200);
  });

  sleep(0.5);
}
