#!/bin/bash
# Disaster Recovery Test Script
# Simulates complete cluster failure and measures RTO

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

# Configuration
NAMESPACE="pixelcore"
BACKUP_NAME="${1:-latest}"
START_TIME=$(date +%s)

# Get latest backup if not specified
if [[ "$BACKUP_NAME" == "latest" ]]; then
    BACKUP_NAME=$(velero backup get --output json | \
        jq -r '.items | sort_by(.status.completionTimestamp) | last | .metadata.name')
fi

log_info "Starting DR test with backup: $BACKUP_NAME"
log_info "This will DELETE the namespace and restore from backup"
read -p "Continue? (yes/no): " confirm
if [[ "$confirm" != "yes" ]]; then
    log_error "DR test cancelled"
    exit 1
fi

# Step 1: Record current state
log_step "Recording current state..."
kubectl get pods -n "$NAMESPACE" -o json > /tmp/dr-test-before.json
PODS_BEFORE=$(kubectl get pods -n "$NAMESPACE" --no-headers | wc -l)
log_info "Pods before: $PODS_BEFORE"

# Step 2: Simulate disaster - delete namespace
log_step "Simulating disaster - deleting namespace..."
DISASTER_TIME=$(date +%s)
kubectl delete namespace "$NAMESPACE" --wait=true
log_info "Namespace deleted"

# Step 3: Restore from backup
log_step "Restoring from backup..."
RESTORE_START=$(date +%s)

velero restore create dr-test-$(date +%Y%m%d-%H%M%S) \
    --from-backup "$BACKUP_NAME" \
    --include-namespaces "$NAMESPACE" \
    --wait

RESTORE_END=$(date +%s)
RESTORE_DURATION=$((RESTORE_END - RESTORE_START))
log_info "Restore completed in ${RESTORE_DURATION}s"

# Step 4: Wait for pods to be ready
log_step "Waiting for pods to be ready..."
RECOVERY_START=$(date +%s)

timeout 600 bash -c "
    while true; do
        ready=\$(kubectl get pods -n $NAMESPACE --no-headers 2>/dev/null | grep -c Running || echo 0)
        total=\$(kubectl get pods -n $NAMESPACE --no-headers 2>/dev/null | wc -l || echo 0)
        echo \"Pods ready: \$ready/\$total\"
        if [[ \$ready -eq \$total && \$total -gt 0 ]]; then
            break
        fi
        sleep 5
    done
"

RECOVERY_END=$(date +%s)
RECOVERY_DURATION=$((RECOVERY_END - RECOVERY_START))

# Step 5: Verify restoration
log_step "Verifying restoration..."

PODS_AFTER=$(kubectl get pods -n "$NAMESPACE" --no-headers | wc -l)
PODS_RUNNING=$(kubectl get pods -n "$NAMESPACE" --no-headers | grep -c Running || echo 0)

log_info "Pods after: $PODS_AFTER (Running: $PODS_RUNNING)"

# Check database
log_step "Checking database..."
if kubectl exec -it postgres-ha-0 -n "$NAMESPACE" -- pg_isready &>/dev/null; then
    log_info "PostgreSQL is ready ✓"
else
    log_error "PostgreSQL is not ready ✗"
fi

# Check Redis
log_step "Checking Redis..."
if kubectl exec -it redis-ha-0 -n "$NAMESPACE" -- redis-cli ping &>/dev/null; then
    log_info "Redis is ready ✓"
else
    log_error "Redis is not ready ✗"
fi

# Calculate RTO
END_TIME=$(date +%s)
TOTAL_RTO=$((END_TIME - DISASTER_TIME))
RTO_MINUTES=$((TOTAL_RTO / 60))

# Results
log_info "========================================="
log_info "DR Test Results"
log_info "========================================="
log_info "Backup used: $BACKUP_NAME"
log_info "Restore duration: ${RESTORE_DURATION}s"
log_info "Recovery duration: ${RECOVERY_DURATION}s"
log_info "Total RTO: ${TOTAL_RTO}s (${RTO_MINUTES} minutes)"
log_info "Pods before: $PODS_BEFORE"
log_info "Pods after: $PODS_AFTER"
log_info "Pods running: $PODS_RUNNING"
log_info "========================================="

# Check if RTO target met
if [[ $RTO_MINUTES -lt 60 ]]; then
    log_info "✓ RTO target met (< 60 minutes)"
else
    log_warn "✗ RTO target NOT met (>= 60 minutes)"
fi

# Save results
cat > /tmp/dr-test-results.json <<EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "backup": "$BACKUP_NAME",
  "restore_duration_seconds": $RESTORE_DURATION,
  "recovery_duration_seconds": $RECOVERY_DURATION,
  "total_rto_seconds": $TOTAL_RTO,
  "total_rto_minutes": $RTO_MINUTES,
  "pods_before": $PODS_BEFORE,
  "pods_after": $PODS_AFTER,
  "pods_running": $PODS_RUNNING,
  "rto_target_met": $([ $RTO_MINUTES -lt 60 ] && echo "true" || echo "false")
}
EOF

log_info "Results saved to /tmp/dr-test-results.json"