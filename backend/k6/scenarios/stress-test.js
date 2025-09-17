import http from 'k6/http';
import { check, sleep } from 'k6';

// Stress test configuration - push system to its limits
export let options = {
  stages: [
    { duration: '2m', target: 200 },  // Ramp up to 200 users
    { duration: '5m', target: 200 },  // Stay at 200 users
    { duration: '2m', target: 400 },  // Ramp up to 400 users
    { duration: '5m', target: 400 },  // Stay at 400 users
    { duration: '2m', target: 600 },  // Ramp up to 600 users
    { duration: '5m', target: 600 },  // Stay at 600 users
    { duration: '5m', target: 0 },    // Ramp down to 0 users
  ],
  thresholds: {
    'http_req_duration': ['p(95)<2000'], // 95% of requests under 2s even under stress
    'http_req_failed': ['rate<0.05'],    // Error rate < 5% under stress
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  const scenario = Math.random();
  
  // Heavy search operations (50% of traffic)
  if (scenario < 0.5) {
    // Complex search with multiple filters
    const params = new URLSearchParams({
      q: 'anime',
      tags: 'action,adventure,fantasy',
      year: '2024',
      status: 'CURRENTLY_AIRING',
      limit: '50'
    });
    
    const res = http.get(`${BASE_URL}/api/search?${params}`);
    check(res, {
      'search handles load': (r) => r.status === 200 || r.status === 503,
      'search doesn't timeout': (r) => r.timings.duration < 5000,
    });
  }
  
  // Concurrent anime details requests (30% of traffic)
  else if (scenario < 0.8) {
    const batch = [];
    for (let i = 0; i < 5; i++) {
      batch.push(['GET', `${BASE_URL}/api/anime/test-${Math.floor(Math.random() * 1000)}`]);
    }
    
    const responses = http.batch(batch);
    responses.forEach(res => {
      check(res, {
        'batch request handled': (r) => r.status === 200 || r.status === 404 || r.status === 503,
      });
    });
  }
  
  // Database-heavy operations (20% of traffic)
  else {
    // Request that requires multiple joins
    const res = http.get(`${BASE_URL}/api/browse/season/2024/SPRING?include_related=true&include_episodes=true`);
    check(res, {
      'heavy query handled': (r) => r.status === 200 || r.status === 503,
      'heavy query doesn't hang': (r) => r.timings.duration < 10000,
    });
  }
  
  sleep(Math.random() * 2); // Shorter sleep for stress test
}