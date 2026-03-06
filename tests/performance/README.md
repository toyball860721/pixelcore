# PixelCore Performance Testing

This directory contains performance testing scripts and tools for PixelCore.

## Overview

Performance testing includes:
- Load testing with k6
- Stress testing
- Endurance testing
- Spike testing
- Performance benchmarking

## Tools

- **k6**: Modern load testing tool
- **Grafana**: Visualization
- **InfluxDB**: Metrics storage

## Quick Start

1. Install k6:
   ```bash
   # macOS
   brew install k6

   # Linux
   sudo apt-get install k6
   ```

2. Run load test:
   ```bash
   k6 run load-test.js
   ```

3. Run stress test:
   ```bash
   k6 run stress-test.js
   ```

## Test Scenarios

- **load-test.js**: Standard load testing
- **stress-test.js**: Stress testing to find limits
- **spike-test.js**: Sudden traffic spike testing
- **endurance-test.js**: Long-duration testing

## Performance Targets

- API P99 latency: < 100ms
- Database query P99: < 50ms
- Cache hit rate: > 95%
- Throughput: > 10,000 RPS

## Documentation

See [PERFORMANCE_TUNING.md](../../docs/PERFORMANCE_TUNING.md) for detailed documentation.
