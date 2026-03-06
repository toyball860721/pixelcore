#!/bin/bash
# Automated chaos experiment runner

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Configuration
NAMESPACE="pixelcore"
EXPERIMENTS_DIR="$(dirname "$0")/experiments"
MONITORING_INTERVAL=10
AVAILABILITY_THRESHOLD=0.99

# Pre-checks
pre_checks() {
    log_step "Running pre-checks..."

    # Check if Chaos Mesh is installed
    if ! kubectl get namespace chaos-mesh &>/dev/null; then
        log_error "Chaos Mesh is not installed. Run install.sh first."
        exit 1
    fi

    # Check if monitoring is active
    if ! kubectl get pods -n monitoring -l app=prometheus | grep -q Running; then
        log_error "Prometheus is not running. Monitoring is required."
        exit 1
    fi

    # Check if target namespace exists
    if ! kubectl get namespace "$NAMESPACE" &>/dev/null; then
        log_error "Namespace $NAMESPACE does not exist"
        exit 1
    fi

    log_info "Pre-checks passed ✓"
}

# Monitor metrics during experiment
monitor_metrics() {
    local experiment_name="$1"
    local duration="$2"

    log_step "Monitoring metrics for $experiment_name (${duration}s)..."

    local start_time=$(date +%s)
    local end_time=$((start_time + duration))

    while [ $(date +%s) -lt $end_time ]; do
        # Check pod status
        local pods_ready=$(kubectl get pods -n "$NAMESPACE" -o json | \
            jq '[.items[] | select(.status.phase=="Running")] | length')
        local pods_total=$(kubectl get pods -n "$NAMESPACE" -o json | jq '.items | length')

        # Check availability
        local availability=$(echo "scale=4; $pods_ready / $pods_total" | bc)

        log_info "Pods: $pods_ready/$pods_total (availability: $availability)"

        if (( $(echo "$availability < $AVAILABILITY_THRESHOLD" | bc -l) )); then
            log_warn "Availability below threshold: $availability < $AVAILABILITY_THRESHOLD"
        fi

        sleep "$MONITORING_INTERVAL"
    done
}

# Run a single experiment
run_experiment() {
    local experiment_file="$1"
    local experiment_name=$(basename "$experiment_file" .yaml)

    log_step "Running experiment: $experiment_name"

    # Apply the experiment (unpause it)
    kubectl apply -f "$experiment_file"

    # Get experiment duration
    local duration=$(kubectl get -f "$experiment_file" -o jsonpath='{.spec.duration}' | sed 's/[^0-9]//g')
    duration=${duration:-60}

    # Monitor during experiment
    monitor_metrics "$experiment_name" "$duration"

    # Wait for experiment to complete
    sleep 5

    # Check if system recovered
    log_step "Checking system recovery..."
    sleep 10

    local pods_ready=$(kubectl get pods -n "$NAMESPACE" -o json | \
        jq '[.items[] | select(.status.phase=="Running")] | length')
    local pods_total=$(kubectl get pods -n "$NAMESPACE" -o json | jq '.items | length')

    if [ "$pods_ready" -eq "$pods_total" ]; then
        log_info "System recovered successfully ✓"
    else
        log_error "System did not fully recover: $pods_ready/$pods_total pods ready"
        return 1
    fi

    # Pause the experiment again
    kubectl patch -f "$experiment_file" --type=merge -p '{"spec":{"paused":true}}'

    log_info "Experiment $experiment_name completed ✓"
}

# Main execution
main() {
    log_info "Starting chaos experiments..."

    pre_checks

    # List of experiments to run
    experiments=(
        "$EXPERIMENTS_DIR/pod-failure.yaml"
        "$EXPERIMENTS_DIR/network-delay.yaml"
        "$EXPERIMENTS_DIR/cpu-stress.yaml"
    )

    local failed=0

    for experiment in "${experiments[@]}"; do
        if [ -f "$experiment" ]; then
            if ! run_experiment "$experiment"; then
                log_error "Experiment failed: $(basename "$experiment")"
                ((failed++))
            fi
            # Wait between experiments
            log_info "Waiting 30s before next experiment..."
            sleep 30
        else
            log_warn "Experiment file not found: $experiment"
        fi
    done

    log_info "========================================="
    if [ $failed -eq 0 ]; then
        log_info "All experiments completed successfully ✓"
        exit 0
    else
        log_error "$failed experiment(s) failed"
        exit 1
    fi
}

# Run main function
main "$@"
