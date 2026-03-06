# Disaster Recovery Runbook

## Overview

This runbook provides step-by-step procedures for recovering from various disaster scenarios. All procedures are designed to achieve RTO (Recovery Time Objective) < 1 hour.

## Prerequisites

- Access to Kubernetes cluster with admin privileges
- Velero CLI installed (`velero version`)
- kubectl configured and authenticated
- Access to backup storage location
- Emergency contact list available

## Disaster Scenarios

### 1. Complete Cluster Failure

**Scenario:** Entire Kubernetes cluster is unavailable or corrupted.

**RTO Target:** 45 minutes

**Procedure:**

1. **Assess the situation** (5 minutes)
   ```bash
   # Check cluster status
   kubectl cluster-info
   kubectl get nodes

   # If cluster is completely down, proceed with recovery
   ```

2. **Provision new cluster** (15 minutes)
   ```bash
   # Use your infrastructure-as-code tool (Terraform, etc.)
   # Or provision manually through cloud provider

   # Verify new cluster is ready
   kubectl get nodes
   kubectl get namespaces
   ```

3. **Install Velero** (5 minutes)
   ```bash
   # Install Velero with same configuration
   velero install \
     --provider aws \
     --plugins velero/velero-plugin-for-aws:v1.9.0 \
     --bucket pixelcore-backups \
     --backup-location-config region=us-west-2 \
     --snapshot-location-config region=us-west-2 \
     --secret-file ./credentials-velero

   # Verify Velero is ready
   kubectl wait --for=condition=Ready pods --all -n velero --timeout=300s
   ```

4. **List available backups** (2 minutes)
   ```bash
   # List all backups
   velero backup get

   # Get latest successful backup
   LATEST_BACKUP=$(velero backup get --output json | \
     jq -r '.items | sort_by(.status.completionTimestamp) | last | .metadata.name')

   echo "Latest backup: $LATEST_BACKUP"

   # Verify backup details
   velero backup describe $LATEST_BACKUP --details
   ```

5. **Restore from backup** (15 minutes)
   ```bash
   # Create restore
   velero restore create full-cluster-restore \
     --from-backup $LATEST_BACKUP \
     --wait

   # Monitor restore progress
   velero restore describe full-cluster-restore

   # Check for errors
   velero restore logs full-cluster-restore
   ```

6. **Verify restoration** (5 minutes)
   ```bash
   # Check all namespaces
   kubectl get namespaces

   # Check pods in pixelcore namespace
   kubectl get pods -n pixelcore

   # Verify all pods are running
   kubectl wait --for=condition=Ready pods --all -n pixelcore --timeout=300s

   # Check services
   kubectl get svc -n pixelcore

   # Test database connectivity
   kubectl exec -it postgres-ha-0 -n pixelcore -- pg_isready

   # Test Redis connectivity
   kubectl exec -it redis-ha-0 -n pixelcore -- redis-cli ping
   ```

7. **Validate application** (3 minutes)
   ```bash
   # Get ingress URL
   INGRESS_URL=$(kubectl get ingress -n pixelcore -o jsonpath='{.items[0].spec.rules[0].host}')

   # Test health endpoint
   curl -f https://$INGRESS_URL/health

   # Check metrics
   curl -f https://$INGRESS_URL/metrics
   ```

**Expected RTO:** 45 minutes

---

### 2. Database Corruption

**Scenario:** PostgreSQL database is corrupted or data is lost.

**RTO Target:** 30 minutes

**Procedure:**

1. **Stop application traffic** (2 minutes)
   ```bash
   # Scale backend to 0 to prevent writes
   kubectl scale deployment backend -n pixelcore --replicas=0
   ```

2. **Identify backup point** (3 minutes)
   ```bash
   # List recent backups
   velero backup get | grep Completed | head -10

   # Choose backup before corruption occurred
   BACKUP_NAME="daily-full-backup-20240306"
   ```

3. **Delete corrupted database** (2 minutes)
   ```bash
   # Delete PostgreSQL StatefulSet
   kubectl delete statefulset postgres-ha -n pixelcore

   # Delete PVCs to remove corrupted data
   kubectl delete pvc -l app=postgres-ha -n pixelcore
   ```

4. **Restore database from backup** (15 minutes)
   ```bash
   # Create selective restore for database only
   velero restore create database-restore \
     --from-backup $BACKUP_NAME \
     --include-resources statefulsets,persistentvolumeclaims \
     --selector app=postgres-ha \
     --wait

   # Wait for database pods to be ready
   kubectl wait --for=condition=Ready pods -l app=postgres-ha -n pixelcore --timeout=600s
   ```

5. **Verify database integrity** (5 minutes)
   ```bash
   # Check PostgreSQL is running
   kubectl exec -it postgres-ha-0 -n pixelcore -- pg_isready

   # Verify replication
   kubectl exec -it postgres-ha-0 -n pixelcore -- \
     psql -U pixelcore -c "SELECT * FROM pg_stat_replication;"
   ```

6. **Restore application traffic** (3 minutes)
   ```bash
   # Scale backend back up
   kubectl scale deployment backend -n pixelcore --replicas=3

   # Wait for pods to be ready
   kubectl wait --for=condition=Ready pods -l app=backend -n pixelcore --timeout=300s
   ```

**Expected RTO:** 30 minutes

---

## Post-Recovery Checklist

- [ ] Verify all services are running
- [ ] Check database integrity and replication
- [ ] Validate application functionality
- [ ] Review monitoring dashboards
- [ ] Check alert status
- [ ] Notify stakeholders
- [ ] Document actual RTO achieved
- [ ] Schedule post-mortem meeting

## Emergency Contacts

- **On-Call Engineer:** [Phone/Slack]
- **Database Admin:** [Phone/Slack]
- **Infrastructure Lead:** [Phone/Slack]

## Related Documents

- [Database Recovery Runbook](./database-recovery.md)
- [Service Degradation Runbook](./service-degradation.md)
