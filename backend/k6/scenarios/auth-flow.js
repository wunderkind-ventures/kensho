import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';

// Test authenticated user flows
export let options = {
  stages: [
    { duration: '1m', target: 50 },   // Ramp up
    { duration: '3m', target: 50 },   // Sustained load
    { duration: '1m', target: 0 },    // Ramp down
  ],
  thresholds: {
    'http_req_duration{type:auth}': ['p(95)<1000'],
    'http_req_duration{type:protected}': ['p(95)<500'],
    'http_req_failed': ['rate<0.02'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

// Test users (would be loaded from a file in production)
const users = new SharedArray('users', function () {
  return [
    { email: 'test1@example.com', password: 'password123' },
    { email: 'test2@example.com', password: 'password123' },
    { email: 'test3@example.com', password: 'password123' },
  ];
});

export default function () {
  const user = users[Math.floor(Math.random() * users.length)];
  let token = null;
  
  // 1. Login
  const loginRes = http.post(
    `${BASE_URL}/api/auth/login`,
    JSON.stringify({
      email: user.email,
      password: user.password,
    }),
    {
      headers: { 'Content-Type': 'application/json' },
      tags: { type: 'auth' },
    }
  );
  
  const loginSuccess = check(loginRes, {
    'login successful': (r) => r.status === 200,
    'login returns token': (r) => {
      if (r.status === 200) {
        const body = JSON.parse(r.body);
        token = body.access_token;
        return token !== undefined;
      }
      return false;
    },
  });
  
  if (!loginSuccess) {
    return; // Skip rest if login failed
  }
  
  sleep(1);
  
  // 2. Make authenticated requests
  const authHeaders = {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
  };
  
  // Get user profile
  const profileRes = http.get(`${BASE_URL}/api/user/profile`, {
    headers: authHeaders,
    tags: { type: 'protected' },
  });
  
  check(profileRes, {
    'profile accessible': (r) => r.status === 200,
  });
  
  sleep(2);
  
  // Request stream URL (authenticated endpoint)
  const streamRes = http.get(`${BASE_URL}/api/stream/episode-123`, {
    headers: authHeaders,
    tags: { type: 'protected' },
  });
  
  check(streamRes, {
    'stream request handled': (r) => r.status === 200 || r.status === 404,
    'stream returns URL if found': (r) => {
      if (r.status === 200) {
        const body = JSON.parse(r.body);
        return body.stream_url !== undefined;
      }
      return true;
    },
  });
  
  sleep(1);
  
  // 3. Token refresh (30% of users)
  if (Math.random() < 0.3) {
    const refreshRes = http.post(
      `${BASE_URL}/api/auth/refresh`,
      JSON.stringify({ token: token }),
      {
        headers: { 'Content-Type': 'application/json' },
        tags: { type: 'auth' },
      }
    );
    
    check(refreshRes, {
      'refresh successful': (r) => r.status === 200,
      'refresh returns new token': (r) => {
        if (r.status === 200) {
          const body = JSON.parse(r.body);
          return body.access_token !== undefined;
        }
        return false;
      },
    });
  }
  
  sleep(3);
  
  // 4. Logout (20% of users)
  if (Math.random() < 0.2) {
    const logoutRes = http.post(
      `${BASE_URL}/api/auth/logout`,
      JSON.stringify({ token: token }),
      {
        headers: authHeaders,
        tags: { type: 'auth' },
      }
    );
    
    check(logoutRes, {
      'logout successful': (r) => r.status === 200,
    });
  }
}