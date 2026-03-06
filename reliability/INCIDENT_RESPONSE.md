# Incident Response Guide

## Incident Management Process

### 1. Detection and Declaration

**Detection Sources:**
- Automated alerts (Prometheus/AlertManager)
- User reports
- Monitoring dashboards
- Health check failures

**Declaration Criteria:**
- Service unavailable or degraded
- Error rate > 1%
- Data loss or corruption
- Security breach
- SLA violation

**Declaration Process:**
```bash
# Create incident channel
/incident declare "Brief description"

# Set severity
/incident severity P0|P1|P2|P3

# Assign incident commander
/incident assign @username
```

---

### 2. Severity Classification

#### P0 - Critical
**Definition:**
- Complete service outage
- Data loss or corruption
- Security breach
- Multiple availability zones down

**Response:**
- Immediate response (< 5 minutes)
- All hands on deck
- Executive notification
- Public status page update

**Examples:**
- Entire cluster down
- Database corruption
- Security breach
- Data center failure

#### P1 - High
**Definition:**
- Partial service degradation
- Single availability zone down
- High error rate (> 5%)
- Database failover

**Response:**
- Response time: < 15 minutes
- On-call engineer + backup
- Team lead notification
- Internal status update

**Examples:**
- Backend pods crash looping
- Database replication failure
- High API latency (> 1s)

#### P2 - Medium
**Definition:**
- Performance degradation
- Non-critical feature failure
- Elevated error rate (1-5%)

**Response:**
- Response time: < 1 hour
- On-call engineer
- Team notification
- Monitor and track

**Examples:**
- Slow queries
- Cache miss rate high
- Single pod failures

#### P3 - Low
**Definition:**
- Minor issues
- Cosmetic bugs
- No user impact

**Response:**
- Response time: Next business day
- Regular team member
- Track in backlog

---

### 3. Response and Mitigation

**Incident Commander Responsibilities:**
- Coordinate response efforts
- Communicate status updates
- Make decisions on mitigation steps
- Escalate if needed

**Response Team:**
- Incident Commander
- On-Call Engineer
- Database Admin (if data layer affected)
- Infrastructure Lead (if cluster affected)

**Initial Response (First 15 minutes):**
1. Acknowledge incident
2. Assess impact and scope
3. Check monitoring dashboards
4. Review recent changes
5. Begin mitigation

**Mitigation Steps:**
```bash
# Check system status
kubectl get pods -n pixelcore
kubectl get nodes
kubectl top nodes

# Check recent events
kubectl get events -n pixelcore --sort-by='.lastTimestamp'

# Check logs
kubectl logs -n pixelcore -l app=backend --tail=100 --since=30m

# Check alerts
kubectl port-forward -n monitoring svc/alertmanager 9093:9093
```

---

### 4. Communication

**Internal Communication:**
- Slack: #incidents channel
- Update frequency: Every 30 minutes
- Include: Status, impact, ETA, next steps

**External Communication:**
- Status page: status.pixelcore.com
- Update frequency: Every hour for P0/P1
- Include: Issue description, impact, progress

**Communication Template:**
```
**Incident Update - [Timestamp]**

Severity: P0/P1/P2/P3
Status: Investigating/Identified/Monitoring/Resolved

Impact:
- [Affected services]
- [User impact]
- [Affected regions]

Current Status:
- [What we know]
- [What we're doing]

Next Update: [Time]
```

---

### 5. Resolution and Recovery

**Resolution Criteria:**
- Service fully restored
- Error rate < 0.1%
- All alerts cleared
- Monitoring shows normal metrics
- User verification complete

**Recovery Steps:**
1. Implement fix
2. Verify fix in staging (if possible)
3. Deploy to production
4. Monitor for 30 minutes
5. Verify all metrics normal
6. Declare resolved

**Post-Resolution:**
```bash
# Verify all services healthy
kubectl get pods -n pixelcore
kubectl get hpa -n pixelcore

# Check metrics
kubectl port-forward -n monitoring svc/prometheus 9090:9090

# Verify no active alerts
kubectl port-forward -n monitoring svc/alertmanager 9093:9093
```

---

### 6. Post-Mortem

**Required For:**
- All P0 incidents
- All P1 incidents
- Any incident with user impact
- Any incident with data loss

**Timeline:**
- Draft: Within 24 hours
- Review: Within 48 hours
- Publish: Within 72 hours

**Post-Mortem Template:**

```markdown
# Incident Post-Mortem: [Title]

## Summary
- Date: [Date]
- Duration: [Start - End]
- Severity: P0/P1/P2/P3
- Impact: [Description]

## Timeline
- [Time] - Incident detected
- [Time] - Incident declared
- [Time] - Root cause identified
- [Time] - Fix implemented
- [Time] - Incident resolved

## Root Cause
[Detailed explanation of what caused the incident]

## Impact
- Users affected: [Number/Percentage]
- Services affected: [List]
- Data loss: [Yes/No - Details]
- Downtime: [Duration]
- SLA impact: [Percentage]

## Resolution
[What was done to resolve the incident]

## What Went Well
- [Positive aspects]

## What Went Wrong
- [Issues encountered]

## Action Items
- [ ] [Action 1] - Owner: [Name] - Due: [Date]
- [ ] [Action 2] - Owner: [Name] - Due: [Date]

## Lessons Learned
[Key takeaways]
```

---

## Escalation Procedures

### Level 1: On-Call Engineer
- Initial response
- Diagnosis and mitigation
- Time limit: 15 minutes

### Level 2: Team Lead + Specialists
- Complex issues
- Multi-service impact
- Time limit: 30 minutes

### Level 3: Management
- P0 incidents
- Extended outages (> 30 minutes)
- Data loss scenarios
- Security incidents

**Escalation Contacts:**
```
On-Call: [Slack/Phone]
Database Admin: [Slack/Phone]
Infrastructure Lead: [Slack/Phone]
Team Lead: [Slack/Phone]
CTO: [Slack/Phone]
VP Engineering: [Slack/Phone]
```

---

## Incident Tools and Resources

### Monitoring
- Prometheus: `kubectl port-forward -n monitoring svc/prometheus 9090:9090`
- Grafana: `kubectl port-forward -n monitoring svc/grafana 3000:3000`
- AlertManager: `kubectl port-forward -n monitoring svc/alertmanager 9093:9093`

### Logs
```bash
# Application logs
kubectl logs -n pixelcore -l app=backend --tail=1000 --since=1h

# Database logs
kubectl logs -n pixelcore -l app=postgres-ha --tail=1000

# Istio logs
kubectl logs -n istio-system -l app=istiod --tail=1000
```

### Runbooks
- [Disaster Recovery](./runbooks/disaster-recovery.md)
- [Database Recovery](./runbooks/database-recovery.md)
- [Service Degradation](./runbooks/service-degradation.md)

---

## Incident Metrics

### Track and Report
- Mean Time to Detect (MTTD)
- Mean Time to Acknowledge (MTTA)
- Mean Time to Resolve (MTTR)
- Incident frequency
- Severity distribution
- SLA impact

### Monthly Review
- Incident trends
- Common root causes
- Action item completion
- Process improvements