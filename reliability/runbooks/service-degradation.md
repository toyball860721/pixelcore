# Service Degradation Runbook

## Overview

This runbook covers procedures for handling partial service failures and graceful degradation scenarios.

## Scenario 1: Backend Service Degradation

### Symptoms
- High error rate (> 1%)
- Increased latency
- Some pods failing health checks

### Procedure

1. **Assess impact**
   ```bash
   # Check pod status
   kubectl get pods -n pixelcore -l app=backend

   # Check recent logs
   kubectl logs -n pixelcore -l app=backend --tail=100 --since=10m

   # Check metrics
   kubectl top pods -n pixelcore -l app=backend
   ```

2. **Scale up if resource constrained**
   ```bash
   # Check HPA status
   kubectl get hpa -n pixelcore

   # Manual scale if needed
   kubectl scale deployment backend -n pixelcore --replicas=5
   ```

3. **Restart unhealthy pods**
   ```bash
   # Delete pods with issues
   kubectl delete pod <pod-name> -n pixelcore

   # Or rolling restart
   kubectl rollout restart deployment backend -n pixelcore
   ```

4. **Enable circuit breaker if cascading**
   ```bash
   # Verify circuit breaker is active
   kubectl get destinationrule backend-circuit-breaker -n pixelcore

   # Check Istio metrics
   kubectl exec -it -n istio-system $(kubectl get pod -n istio-system -l app=istiod -o jsonpath='{.items[0].metadata.name}') -- \
     pilot-agent request GET stats/prometheus | grep circuit
   ```

---

## Scenario 2: Database Performance Issues

### Symptoms
- Slow queries
- High connection count
- Replication lag

### Procedure

1. **Check database metrics**
   ```bash
   # Check connections
   kubectl exec -it postgres-ha-0 -n pixelcore -- \
     psql -U pixelcore -c "SELECT count(*) FROM pg_stat_activity;"

   # Check slow queries
   kubectl exec -it postgres-ha-0 -n pixelcore -- \
     psql -U pixelcore -c "SELECT pid, now() - query_start as duration, query FROM pg_stat_activity WHERE state = 'active' ORDER BY duration DESC;"

   # Check replication lag
   kubectl exec -it postgres-ha-0 -n pixelcore -- \
     psql -U pixelcore -c "SELECT client_addr, state, sync_state, replay_lag FROM pg_stat_replication;"
   ```

2. **Kill long-running queries**
   ```bash
   # Identify problematic query PID
   kubectl exec -it postgres-ha-0 -n pixelcore -- \
     psql -U pixelcore -c "SELECT pg_terminate_backend(<PID>);"
   ```

3. **Route reads to replicas**
   ```bash
   # Update application config to use read service
   # postgres-ha-read:5432 for read-only queries
   ```

---

## Scenario 3: Traffic Rerouting

### Use Istio to route traffic away from degraded services

1. **Route to healthy subset**
   ```bash
   # Create VirtualService to route traffic
   kubectl apply -f - <<EOF
   apiVersion: networking.istio.io/v1beta1
   kind: VirtualService
   metadata:
     name: backend-traffic-split
     namespace: pixelcore
   spec:
     hosts:
       - backend-service
     http:
       - match:
           - headers:
               version:
                 exact: v2
         route:
           - destination:
               host: backend-service
               subset: v2
             weight: 100
       - route:
           - destination:
               host: backend-service
               subset: v1
             weight: 0
   EOF
   ```

2. **Gradual traffic shift**
   ```bash
   # Shift 10% traffic to new version
   kubectl patch virtualservice backend-traffic-split -n pixelcore --type=merge -p '
   spec:
     http:
       - route:
           - destination:
               host: backend-service
               subset: v1
             weight: 90
           - destination:
               host: backend-service
               subset: v2
             weight: 10
   '
   ```

---

## Scenario 4: Graceful Degradation

### Disable non-critical features

1. **Update feature flags**
   ```bash
   # Update ConfigMap to disable features
   kubectl patch configmap pixelcore-config -n pixelcore --type=merge -p '
   data:
     FEATURE_ANALYTICS: "false"
     FEATURE_RECOMMENDATIONS: "false"
   '

   # Restart pods to pick up changes
   kubectl rollout restart deployment backend -n pixelcore
   ```

2. **Reduce resource usage**
   ```bash
   # Lower replica count for non-critical services
   kubectl scale deployment analytics -n pixelcore --replicas=1

   # Reduce HPA max replicas
   kubectl patch hpa backend-hpa -n pixelcore --type=merge -p '
   spec:
     maxReplicas: 5
   '
   ```

---

## Service Dependency Management

### Check service dependencies

```bash
# View service mesh topology
kubectl exec -it -n istio-system $(kubectl get pod -n istio-system -l app=istiod -o jsonpath='{.items[0].metadata.name}') -- \
  pilot-agent request GET debug/config_dump

# Check service connectivity
kubectl run -it --rm debug --image=curlimages/curl --restart=Never -- \
  curl -v http://backend-service.pixelcore:8080/health
```

### Isolate failing dependency

```bash
# Add fault injection to test resilience
kubectl apply -f - <<EOF
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: fault-injection-test
  namespace: pixelcore
spec:
  hosts:
    - external-api-service
  http:
    - fault:
        abort:
          percentage:
            value: 100
          httpStatus: 503
      route:
        - destination:
            host: external-api-service
EOF
```

---

## Monitoring During Degradation

```bash
# Watch pod status
watch kubectl get pods -n pixelcore

# Monitor metrics
kubectl port-forward -n monitoring svc/prometheus 9090:9090
# Open http://localhost:9090

# Check alerts
kubectl port-forward -n monitoring svc/alertmanager 9093:9093
# Open http://localhost:9093
```