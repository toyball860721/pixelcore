# Database Recovery Runbook

## PostgreSQL Point-in-Time Recovery (PITR)

### Scenario: Recover database to specific point in time

**RTO Target:** 30 minutes

**Procedure:**

1. **Identify recovery point**
   ```bash
   # List backups with timestamps
   velero backup get --output json | \
     jq -r '.items[] | "\(.metadata.name) - \(.status.completionTimestamp)"'

   # Choose backup closest to desired recovery point
   BACKUP_NAME="hourly-incremental-backup-20240306-1400"
   ```

2. **Stop application writes**
   ```bash
   kubectl scale deployment backend -n pixelcore --replicas=0
   ```

3. **Restore database**
   ```bash
   # Delete current database
   kubectl delete statefulset postgres-ha -n pixelcore
   kubectl delete pvc -l app=postgres-ha -n pixelcore

   # Restore from backup
   velero restore create pitr-restore \
     --from-backup $BACKUP_NAME \
     --include-resources statefulsets,persistentvolumeclaims,configmaps,secrets \
     --selector app=postgres-ha \
     --wait
   ```

4. **Verify recovery**
   ```bash
   # Wait for pods
   kubectl wait --for=condition=Ready pods -l app=postgres-ha -n pixelcore --timeout=600s

   # Check replication status
   kubectl exec -it postgres-ha-0 -n pixelcore -- \
     psql -U pixelcore -c "SELECT * FROM pg_stat_replication;"

   # Verify data at recovery point
   kubectl exec -it postgres-ha-0 -n pixelcore -- \
     psql -U pixelcore -d pixelcore -c "SELECT NOW();"
   ```

5. **Resume application**
   ```bash
   kubectl scale deployment backend -n pixelcore --replicas=3
   ```

---

## Redis Data Recovery

### Scenario: Recover Redis data from snapshot

**RTO Target:** 15 minutes

**Procedure:**

1. **Stop application**
   ```bash
   kubectl scale deployment backend -n pixelcore --replicas=0
   ```

2. **Restore Redis**
   ```bash
   # Delete current Redis
   kubectl delete statefulset redis-ha -n pixelcore
   kubectl delete pvc -l app=redis-ha -n pixelcore

   # Restore from backup
   velero restore create redis-restore \
     --from-backup $BACKUP_NAME \
     --include-resources statefulsets,persistentvolumeclaims \
     --selector app=redis-ha \
     --wait
   ```

3. **Verify Redis Sentinel**
   ```bash
   # Check Sentinel status
   kubectl exec -it redis-sentinel-0 -n pixelcore -- \
     redis-cli -p 26379 SENTINEL masters

   # Verify master election
   kubectl exec -it redis-sentinel-0 -n pixelcore -- \
     redis-cli -p 26379 SENTINEL get-master-addr-by-name mymaster
   ```

4. **Resume application**
   ```bash
   kubectl scale deployment backend -n pixelcore --replicas=3
   ```

---

## Replication Re-establishment

### PostgreSQL Replication Failure

**Procedure:**

1. **Check replication status**
   ```bash
   kubectl exec -it postgres-ha-0 -n pixelcore -- \
     psql -U pixelcore -c "SELECT * FROM pg_stat_replication;"
   ```

2. **Restart failed replica**
   ```bash
   # Delete failed replica pod
   kubectl delete pod postgres-ha-1 -n pixelcore

   # Wait for pod to restart
   kubectl wait --for=condition=Ready pod/postgres-ha-1 -n pixelcore --timeout=300s
   ```

3. **Verify replication resumed**
   ```bash
   kubectl exec -it postgres-ha-0 -n pixelcore -- \
     psql -U pixelcore -c "SELECT client_addr, state, sync_state FROM pg_stat_replication;"
   ```

### Redis Sentinel Failover

**Procedure:**

1. **Check Sentinel status**
   ```bash
   kubectl exec -it redis-sentinel-0 -n pixelcore -- \
     redis-cli -p 26379 SENTINEL masters
   ```

2. **Force failover if needed**
   ```bash
   kubectl exec -it redis-sentinel-0 -n pixelcore -- \
     redis-cli -p 26379 SENTINEL failover mymaster
   ```

3. **Verify new master**
   ```bash
   kubectl exec -it redis-sentinel-0 -n pixelcore -- \
     redis-cli -p 26379 SENTINEL get-master-addr-by-name mymaster
   ```

---

## Database Integrity Checks

### PostgreSQL

```bash
# Check database size
kubectl exec -it postgres-ha-0 -n pixelcore -- \
  psql -U pixelcore -d pixelcore -c "SELECT pg_size_pretty(pg_database_size('pixelcore'));"

# Check for corrupted indexes
kubectl exec -it postgres-ha-0 -n pixelcore -- \
  psql -U pixelcore -d pixelcore -c "REINDEX DATABASE pixelcore;"

# Vacuum and analyze
kubectl exec -it postgres-ha-0 -n pixelcore -- \
  psql -U pixelcore -d pixelcore -c "VACUUM ANALYZE;"
```

### Redis

```bash
# Check memory usage
kubectl exec -it redis-ha-0 -n pixelcore -- \
  redis-cli -a $REDIS_PASSWORD INFO memory

# Check keyspace
kubectl exec -it redis-ha-0 -n pixelcore -- \
  redis-cli -a $REDIS_PASSWORD INFO keyspace
```