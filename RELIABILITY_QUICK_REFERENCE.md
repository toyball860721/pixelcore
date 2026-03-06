# Reliability Quick Reference

## Emergency Procedures

### Complete Service Outage (P0)
```bash
# 1. Check cluster status
kubectl get nodes
kubectl get pods -n pixelcore

# 2. Check recent events
kubectl get events -n pixelcore --sort-by='.lastTimestamp' | head -20

# 3. Check logs
kubectl logs -n pixelcore -l app=backend --tail=100 --since=30m

# 4. If cluster is down, restore from backup
velero restore create emergency-restore --from-backup $(velero backup get -o json | jq -r '.items | sort_by(.status.completionTimestamp) | last | .metadata.name') --wait

# See: reliability/runbooks/disaster-recovery.md
```

### Database Issues (P1)
```bash
# Check PostgreSQL status
kubectl exec -it postgres-ha-0 -n pixelcore -- pg_isready

# Check replication
kubectl exec -it postgres-ha-0 -n pixelcore -- psql -U pixelcore -c "SELECT * FROM pg_stat_replication;"

# Force failover if needed
kubectl delete pod postgres-ha-0 -n pixelcore

# See: reliability/runbooks/database-recovery.md
```

### High Error Rate (P1)
```bash
# Check backend pods
kubectl get pods -n pixelcore -l app=backend

# Check logs for errors
kubectl logs -n pixelcore -l app=backend --tail=100 | grep -i error

# Scale up if needed
kubectl scale deployment backend -n pixelcore --replicas=5

# Restart unhealthy pods
kubectl delete pod <pod-name> -n pixelcore

# See: reliability/runbooks/service-degradation.md
```

## Common Commands

### Backup & Restore
```bash
# List backups
velero backup get

# Create manual backup
velero backup create manual-backup-$(date +%Y%m%d-%H%M%S) --wait

# Verify backup
./reliability/verify-backup.sh

# Restore from backup
velero restore create --from-backup <backup-name> --wait
```

### Database Operations
```bash
# PostgreSQL health
kubectl exec -it postgres-ha-0 -n pixelcore -- pg_isready

# Check replication lag
kubectl exec -it postgres-ha-0 -n pixelcore -- psql -U pixelcore -c "SELECT client_addr, state, replay_lag FROM pg_stat_replication;"

# Redis health
kubectl exec -it redis-ha-0 -n pixelcore -- redis-cli ping

# Check Redis master
kubectl exec -it redis-sentinel-0 -n pixelcore -- redis-cli -p 26379 SENTINEL get-master-addr-by-name mymaster
```

### Monitoring
```bash
# Port-forward Prometheus
kubectl port-forward -n monitoring svc/prometheus 9090:9090

# Port-forward AlertManager
kubectl port-forward -n monitoring svc/alertmanager 9093:9093

# Port-forward Grafana
kubectl port-forward -n monitoring svc/grafana 3000:3000

# Check active alerts
kubectl exec -it -n monitoring alertmanager-0 -- amtool alert
```

### Service Health
```bash
# Check all pods
kubectl get pods -n pixelcore

# Check HPA status
kubectl get hpa -n pixelcore

# Check PDB status
kubectl get pdb -n pixelcore

# Check circuit breakers
kubectl get destinationrules -n pixelcore

# Top pods by resource usage
kubectl top pods -n pixelcore
```

## Alert Severity Guide

### P0 - Critical (Immediate Response)
- Complete service outage
- Database down
- Data loss
- Security breach

**Action:** Page on-call immediately, escalate to management

### P1 - High (< 15 min response)
- Partial service degradation
- High error rate (> 5%)
- Database failover
- Backup failure

**Action:** On-call engineer responds, notify team

### P2 - Medium (< 1 hour response)
- Performance degradation
- Elevated error rate (1-5%)
- Non-critical feature failure

**Action:** On-call engineer investigates

### P3 - Low (Next business day)
- Minor issues
- Cosmetic bugs
- No user impact

**Action:** Track in backlog

## Key Metrics

### SLA Targets
- Availability: 99.99%
- RTO: < 1 hour
- RPO: < 15 minutes
- API Latency (P99): < 100ms
- Error Rate: < 0.1%

### Health Indicators
```bash
# Availability
kubectl get pods -n pixelcore --no-headers | awk '{if ($3 == "Running") running++; total++} END {print running/total*100 "%"}'

# Error rate (check Prometheus)
# rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])

# Backup status
velero backup get | grep Completed | head -1

# Replication lag
kubectl exec -it postgres-ha-0 -n pixelcore -- psql -U pixelcore -tAc "SELECT COALESCE(MAX(EXTRACT(EPOCH FROM replay_lag)), 0) FROM pg_stat_replication;"
```

## Escalation Contacts

1. **On-Call Engineer** - [Slack/Phone]
2. **Database Admin** - [Slack/Phone]
3. **Infrastructure Lead** - [Slack/Phone]
4. **CTO/VP Engineering** - [Slack/Phone]

## Documentation Links

- **Main Guide:** [docs/RELIABILITY.md](docs/RELIABILITY.md)
- **SLA:** [reliability/SLA.md](reliability/SLA.md)
- **Incident Response:** [reliability/INCIDENT_RESPONSE.md](reliability/INCIDENT_RESPONSE.md)
- **DR Runbook:** [reliability/runbooks/disaster-recovery.md](reliability/runbooks/disaster-recovery.md)
- **Database Runbook:** [reliability/runbooks/database-recovery.md](reliability/runbooks/database-recovery.md)
- **Service Degradation:** [reliability/runbooks/service-degradation.md](reliability/runbooks/service-degradation.md)

## Testing

```bash
# Run reliability test suite
./reliability/tests/reliability-test-suite.sh

# Run DR test (staging only!)
./reliability/scripts/dr-test.sh

# Run chaos experiment
kubectl apply -f reliability/chaos-mesh/experiments/pod-failure.yaml
```

---

**Keep this guide handy for quick reference during incidents!**