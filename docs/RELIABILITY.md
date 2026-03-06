# PixelCore Reliability Guide

## Overview

This document provides a comprehensive guide to the reliability features, practices, and procedures for the PixelCore platform. The system is designed to achieve production-grade reliability with the following SLAs:

- **Availability:** 99.99% (52 minutes downtime/year)
- **RTO (Recovery Time Objective):** < 1 hour
- **RPO (Recovery Point Objective):** < 15 minutes
- **Auto-recovery rate:** > 95%

## Architecture Overview

### High Availability Components

1. **Application Layer**
   - Backend: 3+ replicas with HorizontalPodAutoscaler
   - Frontend: 2+ replicas with HorizontalPodAutoscaler
   - PodDisruptionBudgets ensure minimum availability during disruptions

2. **Data Layer**
   - PostgreSQL: 3-replica HA cluster with streaming replication
   - Redis: 3-replica cluster with Sentinel for automatic failover
   - Synchronous replication ensures data consistency

3. **Service Mesh**
   - Istio for traffic management and resilience
   - Circuit breakers prevent cascading failures
   - Automatic retries for transient failures
   - Load balancing across healthy instances

4. **Monitoring & Alerting**
   - Prometheus for metrics collection
   - AlertManager for alert routing and notification
   - Grafana dashboards for visualization
   - 50+ alert rules covering reliability, availability, and performance

5. **Backup & Recovery**
   - Velero for automated backups
   - Daily full backups (7-day retention)
   - Hourly incremental backups (24-hour retention)
   - 15-minute backups for critical data (4-hour retention)
   - Automated backup verification

## Backup and Recovery

### Backup Strategy

**Backup Schedules:**

1. **Daily Full Backup** (2 AM UTC)
   - All namespaces and cluster resources
   - Retention: 7 days
   - Includes volume snapshots

2. **Hourly Incremental Backup**
   - Critical namespaces (pixelcore, monitoring, istio-system)
   - Retention: 24 hours
   - Deployments, StatefulSets, ConfigMaps, Secrets, PVCs

3. **15-Minute Critical Backup**
   - pixelcore namespace only
   - Retention: 4 hours
   - Achieves RPO < 15 minutes

**Backup Verification:**
- Automated verification runs after each backup
- Checks backup integrity, size, and resource counts
- Alerts on failure via AlertManager

**Restore Procedures:**
- Full cluster restore: ~45 minutes
- Namespace restore: ~20 minutes
- Database PITR: ~30 minutes

See [Disaster Recovery Runbook](../reliability/runbooks/disaster-recovery.md) for detailed procedures.

---

## High Availability Configuration

### PostgreSQL HA

**Architecture:**
- 3 replicas: 1 primary + 2 standby
- Streaming replication with synchronous commit
- Automatic failover via Patroni/pg_auto_failover
- Pod anti-affinity spreads replicas across nodes

**Failover:**
- Automatic promotion of standby < 30 seconds
- Zero data loss with synchronous replication
- Read replicas available via `postgres-ha-read` service

**Configuration:**
```yaml
# Primary: postgres-ha-0.postgres-ha-service
# Replicas: postgres-ha-1, postgres-ha-2
# Read service: postgres-ha-read:5432
```

### Redis HA

**Architecture:**
- 3 Redis replicas: 1 master + 2 slaves
- 3 Redis Sentinel instances for monitoring
- Automatic failover with quorum=2
- Pod anti-affinity for node distribution

**Failover:**
- Sentinel detects master failure < 5 seconds
- Automatic promotion of slave < 30 seconds
- Application reconnects automatically

**Configuration:**
```yaml
# Sentinel service: redis-sentinel-service:26379
# Redis service: redis-ha-service:6379
```

---

## Monitoring and Alerting

### Alert Categories

**Critical Alerts** (immediate notification):
- Service down
- Database connection failure
- Backup failure
- High error rate (> 5%)
- Node not ready
- Pod crash looping

**Warning Alerts** (grouped notification):
- High pod restart rate
- PVC storage almost full (> 85%)
- Database replication lag (> 5s)
- High memory/CPU usage (> 90%)
- Low availability (< 99.99%)

**Info Alerts** (daily digest):
- High request rate
- High network throughput

### Alert Routing

**Channels:**
- Critical: Slack (#alerts-critical) + Email (oncall@example.com)
- Warning: Slack (#alerts-warning)
- Info: Slack (#alerts-info)

**Inhibition Rules:**
- Warning alerts inhibited when critical alert fires
- Pod alerts inhibited when node is down
- Service alerts inhibited when all backends are down

### Dashboards

**Reliability Dashboard:**
- Backup success rate
- RTO/RPO tracking
- Availability metrics (uptime, error rate)
- Database replication lag
- Circuit breaker status
- Error budget tracking

Access: `kubectl port-forward -n monitoring svc/grafana 3000:3000`

---

## Resilience Features

### Circuit Breakers

**Backend Service:**
- Max connections: 100
- Max pending requests: 50
- Consecutive errors threshold: 5
- Ejection time: 30s
- Max ejection: 50%

**Database:**
- Max connections: 50
- Connect timeout: 5s
- Consecutive errors: 3
- Ejection time: 60s

**Redis:**
- Max connections: 100
- Connect timeout: 3s
- Consecutive errors: 5
- Ejection time: 30s

### Retry Policies

**HTTP Requests:**
- Attempts: 3
- Per-try timeout: 2s
- Retry on: 5xx, reset, connect-failure, refused-stream
- Total timeout: 10s

**Database Connections:**
- Automatic reconnection on failure
- Connection pooling with health checks

---

## Chaos Engineering

### Chaos Mesh Experiments

**Pod Failure:**
- Random pod deletion
- Tests HPA scaling and service availability
- Schedule: Weekly (paused by default)

**Network Delay:**
- 100-500ms latency to database
- Tests retry policies and timeout handling
- Schedule: Weekly (paused by default)

**CPU Stress:**
- CPU stress on backend pods
- Tests HPA auto-scaling
- Schedule: Weekly (paused by default)

**Running Experiments:**
```bash
# Install Chaos Mesh
./reliability/chaos-mesh/install.sh

# Run experiment suite
./reliability/chaos-mesh/run-experiments.sh

# Run individual experiment
kubectl apply -f reliability/chaos-mesh/experiments/pod-failure.yaml
```

---

## Incident Response

### Severity Levels

**P0 (Critical):**
- Complete service outage
- Data loss or corruption
- Security breach
- Response time: Immediate
- Escalation: On-call + Management

**P1 (High):**
- Partial service degradation
- Database failover
- High error rate (> 5%)
- Response time: < 15 minutes
- Escalation: On-call engineer

**P2 (Medium):**
- Performance degradation
- Non-critical feature failure
- Response time: < 1 hour
- Escalation: Team lead

**P3 (Low):**
- Minor issues
- Cosmetic bugs
- Response time: Next business day

### Escalation Path

1. On-Call Engineer (0-15 minutes)
2. Database Admin + Infrastructure Lead (15-30 minutes)
3. CTO/VP Engineering (30+ minutes)

### Communication

**Channels:**
- Slack: #incidents
- Email: incidents@example.com
- Status page: status.pixelcore.com

**Templates:**
- Incident declaration
- Status updates (every 30 minutes)
- Resolution notification
- Post-mortem report

---

## Runbooks

Detailed step-by-step procedures for common scenarios:

1. **[Disaster Recovery](../reliability/runbooks/disaster-recovery.md)**
   - Complete cluster failure
   - Database corruption
   - Data center failover

2. **[Database Recovery](../reliability/runbooks/database-recovery.md)**
   - Point-in-time recovery
   - Redis data recovery
   - Replication re-establishment

3. **[Service Degradation](../reliability/runbooks/service-degradation.md)**
   - Backend service issues
   - Database performance problems
   - Traffic rerouting
   - Graceful degradation

---

## SLA Tracking

### Availability Calculation

```
Availability = (Total Time - Downtime) / Total Time × 100%
```

**99.99% Availability:**
- Monthly: 4.38 minutes downtime
- Yearly: 52.56 minutes downtime

**Error Budget:**
- Monthly: 4.38 minutes
- Consumed by: Incidents, deployments, maintenance

### Monitoring SLA Compliance

```bash
# Check current availability
kubectl exec -it -n monitoring prometheus-0 -- \
  promtool query instant 'http://localhost:9090' \
  'sum(rate(http_requests_total{status!~"5.."}[30d])) / sum(rate(http_requests_total[30d]))'

# Check error budget remaining
# (Tracked in Grafana dashboard)
```

---

## Testing and Validation

### DR Drill Schedule

- **Full DR Drill:** Quarterly
- **Database Recovery Test:** Monthly
- **Backup Verification:** Daily (automated)
- **Chaos Experiments:** Weekly (manual)

### Test Checklist

- [ ] Backup and restore test
- [ ] Database failover test
- [ ] Circuit breaker validation
- [ ] Alert notification test
- [ ] Runbook walkthrough
- [ ] Performance under failure

### Validation Script

```bash
# Run comprehensive reliability test suite
./reliability/tests/reliability-test-suite.sh
```

---

## Maintenance Windows

**Scheduled Maintenance:**
- Day: Sunday
- Time: 2-4 AM UTC
- Frequency: Monthly
- Notification: 7 days advance

**Emergency Maintenance:**
- Approval: CTO/VP Engineering
- Notification: Immediate
- Post-mortem: Required

---

## Related Documents

- [SLA Definitions](../reliability/SLA.md)
- [Incident Response Guide](../reliability/INCIDENT_RESPONSE.md)
- [Disaster Recovery Runbook](../reliability/runbooks/disaster-recovery.md)
- [Database Recovery Runbook](../reliability/runbooks/database-recovery.md)
- [Service Degradation Runbook](../reliability/runbooks/service-degradation.md)

---

## Contact Information

**On-Call Rotation:**
- Primary: [Slack/Phone]
- Secondary: [Slack/Phone]

**Team Leads:**
- Infrastructure: [Contact]
- Database: [Contact]
- Platform: [Contact]

**Emergency Escalation:**
- CTO: [Contact]
- VP Engineering: [Contact]
