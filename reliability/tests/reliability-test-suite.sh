#!/bin/bash
# Comprehensive Reliability Test Suite

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

NAMESPACE="pixelcore"
TESTS_PASSED=0
TESTS_FAILED=0

run_test() {
    local test_name="$1"
    local test_command="$2"

    log_step "Running: $test_name"
    if eval "$test_command"; then
        log_info "✓ PASSED: $test_name"
        ((TESTS_PASSED++))
        return 0
    else
        log_error "✗ FAILED: $test_name"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Test 1: Backup Verification
test_backup_verification() {
    log_step "Test 1: Backup Verification"

    # Check if backups exist
    local backup_count=$(velero backup get --output json | jq '.items | length')
    if [[ $backup_count -gt 0 ]]; then
        log_info "Found $backup_count backups"

        # Check latest backup
        local latest_backup=$(velero backup get --output json | \
            jq -r '.items | sort_by(.status.completionTimestamp) | last | .metadata.name')
        local backup_phase=$(velero backup describe "$latest_backup" --output json | jq -r '.phase')

        if [[ "$backup_phase" == "Completed" ]]; then
            log_info "Latest backup ($latest_backup) is Completed"
            return 0
        else
            log_error "Latest backup phase: $backup_phase"
            return 1
        fi
    else
        log_error "No backups found"
        return 1
    fi
}

# Test 2: Database HA
test_database_ha() {
    log_step "Test 2: Database High Availability"

    # Check PostgreSQL replicas
    local pg_replicas=$(kubectl get statefulset postgres-ha -n "$NAMESPACE" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo 0)
    local pg_ready=$(kubectl get statefulset postgres-ha -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo 0)

    if [[ $pg_replicas -ge 3 && $pg_ready -eq $pg_replicas ]]; then
        log_info "PostgreSQL HA: $pg_ready/$pg_replicas replicas ready"

        # Check replication
        if kubectl exec -it postgres-ha-0 -n "$NAMESPACE" -- \
            psql -U pixelcore -tAc "SELECT count(*) FROM pg_stat_replication;" 2>/dev/null | grep -q "2"; then
            log_info "PostgreSQL replication active"
            return 0
        else
            log_error "PostgreSQL replication not active"
            return 1
        fi
    else
        log_error "PostgreSQL HA not ready: $pg_ready/$pg_replicas"
        return 1
    fi
}

# Test 3: Redis HA
test_redis_ha() {
    log_step "Test 3: Redis High Availability"

    # Check Redis replicas
    local redis_replicas=$(kubectl get statefulset redis-ha -n "$NAMESPACE" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo 0)
    local redis_ready=$(kubectl get statefulset redis-ha -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo 0)

    # Check Sentinel
    local sentinel_replicas=$(kubectl get statefulset redis-sentinel -n "$NAMESPACE" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo 0)
    local sentinel_ready=$(kubectl get statefulset redis-sentinel -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo 0)

    if [[ $redis_replicas -ge 3 && $redis_ready -eq $redis_replicas && \
          $sentinel_replicas -ge 3 && $sentinel_ready -eq $sentinel_replicas ]]; then
        log_info "Redis HA: $redis_ready/$redis_replicas replicas ready"
        log_info "Sentinel: $sentinel_ready/$sentinel_replicas replicas ready"
        return 0
    else
        log_error "Redis HA not ready"
        return 1
    fi
}

# Test 4: Prometheus Alerts
test_prometheus_alerts() {
    log_step "Test 4: Prometheus Alert Rules"

    # Check if Prometheus is running
    if ! kubectl get pods -n monitoring -l app=prometheus | grep -q Running; then
        log_error "Prometheus not running"
        return 1
    fi

    # Check alert rules loaded
    local rules_count=$(kubectl exec -it -n monitoring prometheus-0 -- \
        wget -qO- http://localhost:9090/api/v1/rules 2>/dev/null | \
        jq '.data.groups | length' || echo 0)

    if [[ $rules_count -gt 0 ]]; then
        log_info "Prometheus has $rules_count rule groups loaded"
        return 0
    else
        log_error "No Prometheus rules loaded"
        return 1
    fi
}

# Test 5: Circuit Breakers
test_circuit_breakers() {
    log_step "Test 5: Circuit Breakers"

    # Check DestinationRules exist
    local dr_count=$(kubectl get destinationrules -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l)

    if [[ $dr_count -gt 0 ]]; then
        log_info "Found $dr_count DestinationRules"

        # Check backend circuit breaker
        if kubectl get destinationrule backend-circuit-breaker -n "$NAMESPACE" &>/dev/null; then
            log_info "Backend circuit breaker configured"
            return 0
        else
            log_warn "Backend circuit breaker not found"
            return 1
        fi
    else
        log_error "No DestinationRules found"
        return 1
    fi
}

# Test 6: PodDisruptionBudgets
test_pdbs() {
    log_step "Test 6: PodDisruptionBudgets"

    local pdb_count=$(kubectl get pdb -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l)

    if [[ $pdb_count -ge 4 ]]; then
        log_info "Found $pdb_count PodDisruptionBudgets"
        return 0
    else
        log_error "Insufficient PodDisruptionBudgets: $pdb_count (expected >= 4)"
        return 1
    fi
}

# Test 7: HPA Configuration
test_hpa() {
    log_step "Test 7: HorizontalPodAutoscaler"

    local hpa_count=$(kubectl get hpa -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l)

    if [[ $hpa_count -ge 2 ]]; then
        log_info "Found $hpa_count HPAs"

        # Check backend HPA
        local backend_replicas=$(kubectl get hpa backend-hpa -n "$NAMESPACE" -o jsonpath='{.status.currentReplicas}' 2>/dev/null || echo 0)
        if [[ $backend_replicas -ge 2 ]]; then
            log_info "Backend HPA: $backend_replicas replicas"
            return 0
        else
            log_error "Backend HPA has insufficient replicas: $backend_replicas"
            return 1
        fi
    else
        log_error "Insufficient HPAs: $hpa_count"
        return 1
    fi
}

# Test 8: Monitoring Stack
test_monitoring() {
    log_step "Test 8: Monitoring Stack"

    # Check Prometheus
    if kubectl get pods -n monitoring -l app=prometheus | grep -q Running; then
        log_info "Prometheus running"
    else
        log_error "Prometheus not running"
        return 1
    fi

    # Check AlertManager
    if kubectl get pods -n monitoring -l app=alertmanager | grep -q Running; then
        log_info "AlertManager running"
    else
        log_warn "AlertManager not running"
    fi

    return 0
}

# Main execution
main() {
    log_info "========================================="
    log_info "Reliability Test Suite"
    log_info "========================================="

    run_test "Backup Verification" "test_backup_verification"
    run_test "Database High Availability" "test_database_ha"
    run_test "Redis High Availability" "test_redis_ha"
    run_test "Prometheus Alert Rules" "test_prometheus_alerts"
    run_test "Circuit Breakers" "test_circuit_breakers"
    run_test "PodDisruptionBudgets" "test_pdbs"
    run_test "HorizontalPodAutoscaler" "test_hpa"
    run_test "Monitoring Stack" "test_monitoring"

    log_info "========================================="
    log_info "Test Results"
    log_info "========================================="
    log_info "Passed: $TESTS_PASSED"
    log_info "Failed: $TESTS_FAILED"
    log_info "Total: $((TESTS_PASSED + TESTS_FAILED))"

    if [[ $TESTS_FAILED -eq 0 ]]; then
        log_info "✓ All tests passed!"
        exit 0
    else
        log_error "✗ Some tests failed"
        exit 1
    fi
}

main "$@"