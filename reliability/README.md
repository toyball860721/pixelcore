# Reliability Implementation

This directory contains all reliability-related configurations, scripts, and documentation for the PixelCore platform.

## Quick Start

### 1. Deploy Reliability Features

```bash
# Deploy database HA
kubectl apply -f ../k8s/base/postgres-ha.yaml
kubectl apply -f ../k8s/base/postgres-pdb.yaml
kubectl apply -f ../k8s/base/redis-ha.yaml
kubectl apply -f ../k8s/base/redis-pdb.yaml

# Deploy Velero backup schedules
kubectl apply -f velero-schedules.yaml
kubectl apply -f backup-storage-class.yaml

# Deploy monitoring alerts
kubectl apply -f ../monitoring/alerts/reliability-rules.yaml
kubectl apply -f ../monitoring/alerts/availability-rules.yaml
kubectl apply -f ../monitoring/alerts/performance-rules.yaml
kubectl apply -f ../monitoring/alertmanager-config.yaml

# Deploy circuit breakers and retry policies
kubectl apply -f ../k8s/service-mesh/destination-rules/circuit-breakers.yaml
kubectl apply -f ../k8s/service-mesh/virtual-services/retry-policies.yaml
```

### 2. Install Chaos Mesh (Optional)

```bash
./chaos-mesh/install.sh
```

### 3. Run Validation Tests

```bash
# Run comprehensive test suite
./tests/reliability-test-suite.sh

# Run DR test (WARNING: Destructive in staging)
./scripts/dr-test.sh
```

## Directory Structure

```
reliability/
├── README.md                          # This file
├── SLA.md                             # Service Level Agreements
├── INCIDENT_RESPONSE.md               # Incident management guide
├── validation-checklist.md            # Pre-production checklist
├── velero-schedules.yaml              # Automated backup schedules
├── backup-storage-class.yaml          # Storage configuration
├── restore-procedures.yaml            # Velero restore templates
├── verify-backup.sh                   # Automated backup verification
├── chaos-mesh/
│   ├── install.sh                     # Chaos Mesh installation
│   ├── run-experiments.sh             # Automated experiment runner
│   └── experiments/
│       ├── pod-failure.yaml           # Pod failure chaos
│       ├── network-delay.yaml         # Network latency chaos
│       └── cpu-stress.yaml            # CPU stress chaos
├── runbooks/
│   ├── disaster-recovery.md           # DR procedures
│   ├── database-recovery.md           # Database PITR
│   └── service-degradation.md         # Partial failure handling
├── scripts/
│   └── dr-test.sh                     # DR drill automation
└── tests/
    └── reliability-test-suite.sh      # Test suite
```

## Key Features

### 1. Automated Backups (RPO < 15 minutes)
- Daily full backup: 2 AM UTC, 7-day retention
- Hourly incremental: 24-hour retention
- 15-minute critical: 4-hour retention
- Automated verification after each backup

### 2. Database High Availability
- PostgreSQL: 3 replicas with streaming replication
- Redis: 3 replicas with Sentinel (quorum=2)
- Automatic failover: < 30 seconds
- Zero data loss with synchronous replication

### 3. Monitoring & Alerting
- 50+ alert rules covering reliability, availability, performance
- AlertManager with Slack + email notifications
- Grafana dashboard for real-time metrics
- SLA tracking for 99.99% availability

### 4. Resilience Features
- Circuit breakers to prevent cascading failures
- Retry policies for transient failures
- PodDisruptionBudgets ensure minimum availability
- HorizontalPodAutoscaler for auto-scaling

### 5. Chaos Engineering
- Pod failure tests
- Network delay tests
- CPU stress tests
- Automated experiment runner

## SLA Targets

- **Availability:** 99.99% (52 minutes downtime/year)
- **RTO:** < 1 hour
- **RPO:** < 15 minutes
- **API Latency:** P99 < 100ms
- **Error Rate:** < 0.1%

## Common Operations

### Verify Backup
```bash
velero backup get
./verify-backup.sh
```

### Test Database Failover
```bash
kubectl delete pod postgres-ha-0 -n pixelcore
watch kubectl get pods -n pixelcore -l app=postgres-ha
```

### Test Redis Failover
```bash
kubectl delete pod redis-ha-0 -n pixelcore
kubectl exec -it redis-sentinel-0 -n pixelcore -- \
  redis-cli -p 26379 SENTINEL get-master-addr-by-name mymaster
```

### Run Chaos Experiment
```bash
kubectl apply -f chaos-mesh/experiments/pod-failure.yaml
watch kubectl get pods -n pixelcore
```

## Runbooks

1. **[Disaster Recovery](runbooks/disaster-recovery.md)** - Complete cluster failure, database corruption, data center failover
2. **[Database Recovery](runbooks/database-recovery.md)** - PostgreSQL PITR, Redis recovery, replication
3. **[Service Degradation](runbooks/service-degradation.md)** - Partial failures, traffic rerouting, graceful degradation

## Testing Schedule

- **Daily:** Automated backup verification
- **Weekly:** Backup restore test
- **Monthly:** Database/Redis failover tests
- **Quarterly:** Full DR drill, Chaos experiments

## Documentation

- **[RELIABILITY.md](../docs/RELIABILITY.md)** - Comprehensive guide
- **[SLA.md](SLA.md)** - Service Level Agreements
- **[INCIDENT_RESPONSE.md](INCIDENT_RESPONSE.md)** - Incident management
- **[validation-checklist.md](validation-checklist.md)** - Pre-production checklist

## Emergency Contacts

- On-Call Engineer: [Slack/Phone]
- Database Admin: [Slack/Phone]
- Infrastructure Lead: [Slack/Phone]
- CTO/VP Engineering: [Slack/Phone]