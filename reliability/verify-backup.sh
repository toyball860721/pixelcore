#!/bin/bash
# Automated backup verification script
# Runs after each backup completion to validate integrity

set -euo pipefail

# Configuration
BACKUP_NAME="${1:-latest}"
ALERT_WEBHOOK="${ALERT_WEBHOOK:-}"
MIN_BACKUP_SIZE_MB=10

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

send_alert() {
    local message="$1"
    if [[ -n "$ALERT_WEBHOOK" ]]; then
        curl -X POST "$ALERT_WEBHOOK" \
            -H 'Content-Type: application/json' \
            -d "{\"text\":\"Backup Verification Failed: $message\"}" \
            2>/dev/null || true
    fi
}

# Get latest backup if not specified
if [[ "$BACKUP_NAME" == "latest" ]]; then
    BACKUP_NAME=$(velero backup get --output json | \
        jq -r '.items | sort_by(.status.completionTimestamp) | last | .metadata.name')

    if [[ -z "$BACKUP_NAME" || "$BACKUP_NAME" == "null" ]]; then
        log_error "No backups found"
        send_alert "No backups found in Velero"
        exit 1
    fi
fi

log_info "Verifying backup: $BACKUP_NAME"

# Get backup details
BACKUP_JSON=$(velero backup describe "$BACKUP_NAME" --details --output json)

# Check backup phase
PHASE=$(echo "$BACKUP_JSON" | jq -r '.phase')
if [[ "$PHASE" != "Completed" ]]; then
    log_error "Backup phase is $PHASE (expected: Completed)"
    send_alert "Backup $BACKUP_NAME failed with phase: $PHASE"
    exit 1
fi
log_info "Backup phase: $PHASE ✓"

# Check for errors
ERRORS=$(echo "$BACKUP_JSON" | jq -r '.errors // 0')
if [[ "$ERRORS" -gt 0 ]]; then
    log_error "Backup has $ERRORS errors"
    send_alert "Backup $BACKUP_NAME completed with $ERRORS errors"
    exit 1
fi
log_info "No errors found ✓"

# Check for warnings
WARNINGS=$(echo "$BACKUP_JSON" | jq -r '.warnings // 0')
if [[ "$WARNINGS" -gt 0 ]]; then
    log_warn "Backup has $WARNINGS warnings"
fi

# Validate backup size
BACKUP_SIZE_BYTES=$(echo "$BACKUP_JSON" | jq -r '.status.progress.totalBytes // 0')
BACKUP_SIZE_MB=$((BACKUP_SIZE_BYTES / 1024 / 1024))

if [[ "$BACKUP_SIZE_MB" -lt "$MIN_BACKUP_SIZE_MB" ]]; then
    log_error "Backup size ($BACKUP_SIZE_MB MB) is below minimum ($MIN_BACKUP_SIZE_MB MB)"
    send_alert "Backup $BACKUP_NAME size is suspiciously small: $BACKUP_SIZE_MB MB"
    exit 1
fi
log_info "Backup size: $BACKUP_SIZE_MB MB ✓"

# Check resource counts
ITEMS_BACKED_UP=$(echo "$BACKUP_JSON" | jq -r '.status.progress.itemsBackedUp // 0')
if [[ "$ITEMS_BACKED_UP" -eq 0 ]]; then
    log_error "No items were backed up"
    send_alert "Backup $BACKUP_NAME contains 0 items"
    exit 1
fi
log_info "Items backed up: $ITEMS_BACKED_UP ✓"

# Check volume snapshots
VOLUME_SNAPSHOTS=$(echo "$BACKUP_JSON" | jq -r '.status.volumeSnapshotsCompleted // 0')
log_info "Volume snapshots: $VOLUME_SNAPSHOTS"

# Check expiration
EXPIRATION=$(echo "$BACKUP_JSON" | jq -r '.status.expiration')
log_info "Backup expires: $EXPIRATION"

# Summary
log_info "========================================="
log_info "Backup verification PASSED ✓"
log_info "Backup: $BACKUP_NAME"
log_info "Size: $BACKUP_SIZE_MB MB"
log_info "Items: $ITEMS_BACKED_UP"
log_info "Snapshots: $VOLUME_SNAPSHOTS"
log_info "========================================="

exit 0
