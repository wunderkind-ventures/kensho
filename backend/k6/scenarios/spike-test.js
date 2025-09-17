import http from 'k6/http';
import { check } from 'k6';

// Spike test - sudden traffic surge simulation
export let options = {
  stages: [
    { duration: '30s', target: 50 },    // Warm up
    { duration: '1m', target: 50 },     // Stay at 50
    { duration: '30s', target: 1000 },  // Spike to 1000 users!
    { duration: '3m', target: 1000 },   // Stay at 1000
    { duration: '30s', target: 50 },    // Scale down
    { duration: '2m', target: 50 },     // Recovery period
    { duration: '30s', target: 0 },     // Ramp down
  ],
  thresholds: {
    'http_req_duration': ['p(95)<3000'], // Allow higher latency during spike
    'http_req_failed': ['rate<0.1'],     // Allow up to 10% errors during spike
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  // Simulate typical user behavior during a spike
  // (e.g., new episode release, trending anime)
  
  const requests = [
    {
      method: 'GET',
      url: `${BASE_URL}/api/search?q=trending`,
      params: { tags: ['check trending search'] }
    },
    {
      method: 'GET',
      url: `${BASE_URL}/api/browse/season/2024/FALL`,
      params: { tags: ['check seasonal browse'] }
    },
    {
      method: 'GET',
      url: `${BASE_URL}/api/anime/popular-anime-id`,
      params: { tags: ['check popular anime'] }
    }
  ];
  
  const req = requests[Math.floor(Math.random() * requests.length)];
  const res = http.request(req.method, req.url);
  
  check(res, {
    'spike response handled': (r) => r.status === 200 || r.status === 404 || r.status === 503,
    'spike doesn\'t cause crash': (r) => r.status !== 500,
  }, req.params);
}