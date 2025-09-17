# K6 Load Testing Suite

## Overview
Performance and load testing suite for Project Kenshō API using k6.

## Prerequisites
```bash
# Install k6
brew install k6  # macOS
# or
curl https://github.com/grafana/k6/releases/download/v0.47.0/k6-v0.47.0-linux-amd64.tar.gz -L | tar xvz
```

## Test Scenarios

### 1. Normal Load Test (`normal-load.js`)
Simulates typical user behavior with realistic traffic patterns.
- **Users**: Ramps up to 100 concurrent users
- **Duration**: 9 minutes total
- **Scenarios**: Search (40%), Browse (30%), Episodes (30%)
- **Thresholds**: P95 < 500ms, Error rate < 1%

### 2. Stress Test (`stress-test.js`)
Pushes the system to identify breaking points.
- **Users**: Gradually increases to 600 concurrent users
- **Duration**: 26 minutes total
- **Focus**: Heavy queries, batch requests, complex searches
- **Thresholds**: P95 < 2s, Error rate < 5%

### 3. Spike Test (`spike-test.js`)
Simulates sudden traffic surges (e.g., new episode release).
- **Users**: Sudden spike from 50 to 1000 users
- **Duration**: 7.5 minutes total
- **Purpose**: Test auto-scaling and recovery
- **Thresholds**: P95 < 3s, Error rate < 10%

### 4. Auth Flow Test (`auth-flow.js`)
Tests authentication and protected endpoints.
- **Users**: 50 authenticated users
- **Duration**: 5 minutes
- **Scenarios**: Login, protected requests, token refresh, logout
- **Thresholds**: Auth P95 < 1s, Protected P95 < 500ms

## Running Tests

### Basic Usage
```bash
# Run normal load test
k6 run k6/scenarios/normal-load.js

# Run with custom base URL
k6 run -e BASE_URL=https://api.kensho.example.com k6/scenarios/normal-load.js

# Run stress test
k6 run k6/scenarios/stress-test.js

# Run spike test
k6 run k6/scenarios/spike-test.js

# Run auth flow test
k6 run k6/scenarios/auth-flow.js
```

### Advanced Options
```bash
# Output results to JSON
k6 run --out json=results.json k6/scenarios/normal-load.js

# Send metrics to InfluxDB
k6 run --out influxdb=http://localhost:8086/k6 k6/scenarios/normal-load.js

# Run with more VUs (virtual users)
k6 run --vus 200 --duration 10m k6/scenarios/normal-load.js

# Run in cloud (requires k6 cloud account)
k6 cloud k6/scenarios/normal-load.js
```

## Interpreting Results

### Key Metrics
- **http_req_duration**: Response time (aim for P95 < 500ms)
- **http_req_failed**: Failed request rate (aim for < 1%)
- **http_reqs**: Requests per second (throughput)
- **vus**: Active virtual users
- **iterations**: Completed test iterations

### Example Output
```
     ✓ search status is 200
     ✓ search returns results
     ✓ search response time < 200ms

     checks.........................: 98.52% ✓ 14523  ✗ 218
     data_received..................: 438 MB 1.5 MB/s
     data_sent......................: 87 MB  291 kB/s
     http_req_blocked...............: avg=2.3ms    p(95)=11ms
     http_req_duration..............: avg=142.3ms  p(95)=487ms
       { type:auth }...............: avg=523.1ms  p(95)=982ms
       { type:protected }...........: avg=87.2ms   p(95)=234ms
     http_req_failed................: 0.73%  ✓ 156    ✗ 21234
     http_reqs......................: 21390  71.3/s
     iterations.....................: 7130   23.77/s
     vus............................: 1      min=1    max=100
     vus_max........................: 100    min=100  max=100
```

## CI/CD Integration

### GitHub Actions
```yaml
- name: Run load tests
  run: |
    # Install k6
    sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
    echo "deb https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
    sudo apt-get update
    sudo apt-get install k6
    
    # Run tests
    k6 run --quiet k6/scenarios/normal-load.js
```

### Performance Gates
```javascript
// Add to test file for CI/CD gates
export let options = {
  thresholds: {
    'http_req_duration': [
      { threshold: 'p(95)<500', abortOnFail: true },  // Abort if P95 > 500ms
    ],
    'http_req_failed': [
      { threshold: 'rate<0.01', abortOnFail: true },  // Abort if errors > 1%
    ],
  },
};
```

## Monitoring Integration

### Grafana Dashboard
1. Import dashboard from `k6/dashboards/k6-dashboard.json`
2. Configure InfluxDB data source
3. Run tests with InfluxDB output

### Prometheus Metrics
```bash
# Run k6 with Prometheus remote write
k6 run -o experimental-prometheus-rw \
  --tag testid=normal-load-$(date +%s) \
  k6/scenarios/normal-load.js
```

## Best Practices

1. **Baseline First**: Run tests against current version to establish baseline
2. **Incremental Load**: Start with small loads, gradually increase
3. **Monitor Resources**: Watch CPU, memory, database connections during tests
4. **Test in Staging**: Always test in staging environment that mirrors production
5. **Regular Testing**: Run tests as part of CI/CD pipeline
6. **Document Results**: Keep historical data for trend analysis

## Troubleshooting

### Common Issues

1. **Connection Refused**
   - Ensure backend is running: `cargo run --bin backend-server`
   - Check BASE_URL environment variable

2. **High Error Rate**
   - Check backend logs for errors
   - Verify database is accessible
   - Check for rate limiting

3. **Slow Response Times**
   - Check database query performance
   - Verify caching is working
   - Monitor network latency

### Debug Mode
```bash
# Run with HTTP debug output
k6 run --http-debug k6/scenarios/normal-load.js

# Run with verbose logging
k6 run --verbose k6/scenarios/normal-load.js
```

## Performance Targets

Based on the implementation roadmap:
- ✅ P95 latency < 200ms (with cache)
- ✅ P99 latency < 500ms
- ✅ Cache hit ratio > 80%
- ✅ < 1% error rate under normal load
- ✅ Recovery time < 30s after spike

---

*Part of Project Kenshō - T067: Load Testing Implementation*