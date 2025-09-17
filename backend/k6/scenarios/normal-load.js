import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');

// Test configuration
export let options = {
  stages: [
    { duration: '2m', target: 100 }, // Ramp up to 100 users
    { duration: '5m', target: 100 }, // Stay at 100 users
    { duration: '2m', target: 0 },   // Ramp down to 0 users
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500', 'p(99)<1000'], // 95% < 500ms, 99% < 1s
    'http_req_failed': ['rate<0.01'],                  // Error rate < 1%
    'errors': ['rate<0.01'],                           // Custom error rate < 1%
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

// Test data
const searchTerms = [
  'spy family',
  'demon slayer',
  'jujutsu',
  'one piece',
  'attack on titan',
  'chainsaw man',
  'tokyo ghoul',
  'naruto',
  'bleach',
  'death note'
];

const seasons = [
  { year: 2024, season: 'SPRING' },
  { year: 2024, season: 'SUMMER' },
  { year: 2024, season: 'FALL' },
  { year: 2023, season: 'WINTER' }
];

export default function () {
  // Scenario 1: Search for anime (40% of traffic)
  if (Math.random() < 0.4) {
    const searchTerm = searchTerms[Math.floor(Math.random() * searchTerms.length)];
    const searchRes = http.get(`${BASE_URL}/api/search?q=${encodeURIComponent(searchTerm)}`);
    
    const searchSuccess = check(searchRes, {
      'search status is 200': (r) => r.status === 200,
      'search returns results': (r) => {
        const body = JSON.parse(r.body);
        return Array.isArray(body.results);
      },
      'search response time < 200ms': (r) => r.timings.duration < 200,
    });
    
    errorRate.add(!searchSuccess);
    
    // If search successful, get details of first result
    if (searchSuccess && searchRes.json('results').length > 0) {
      const animeId = searchRes.json('results')[0].id;
      sleep(1);
      
      const detailsRes = http.get(`${BASE_URL}/api/anime/${animeId}`);
      check(detailsRes, {
        'anime details status is 200': (r) => r.status === 200,
        'anime details has title': (r) => JSON.parse(r.body).title !== undefined,
      });
    }
  }
  
  // Scenario 2: Browse seasonal anime (30% of traffic)
  else if (Math.random() < 0.7) {
    const season = seasons[Math.floor(Math.random() * seasons.length)];
    const browseRes = http.get(`${BASE_URL}/api/browse/season/${season.year}/${season.season}`);
    
    const browseSuccess = check(browseRes, {
      'browse status is 200': (r) => r.status === 200,
      'browse returns anime list': (r) => {
        const body = JSON.parse(r.body);
        return Array.isArray(body.anime);
      },
      'browse response time < 300ms': (r) => r.timings.duration < 300,
    });
    
    errorRate.add(!browseSuccess);
  }
  
  // Scenario 3: Get anime episodes (30% of traffic)
  else {
    // Use a known anime ID (would be from previous search in real scenario)
    const animeId = 'test-anime-' + Math.floor(Math.random() * 100);
    const episodesRes = http.get(`${BASE_URL}/api/anime/${animeId}/episodes`);
    
    const episodesSuccess = check(episodesRes, {
      'episodes status is 200 or 404': (r) => r.status === 200 || r.status === 404,
      'episodes response time < 250ms': (r) => r.timings.duration < 250,
    });
    
    if (episodesRes.status === 200) {
      check(episodesRes, {
        'episodes returns list': (r) => {
          const body = JSON.parse(r.body);
          return Array.isArray(body.episodes);
        },
      });
    }
    
    errorRate.add(!episodesSuccess && episodesRes.status !== 404);
  }
  
  sleep(Math.random() * 3 + 1); // Random sleep between 1-4 seconds
}