# PixelCore Security

This directory contains security scanning and hardening configurations for PixelCore.

## Overview

Security measures include:
- Container vulnerability scanning with Trivy
- Web application security testing with OWASP ZAP
- Secret management with Vault
- Security policies and compliance

## Tools

- **Trivy**: Container and dependency scanning
- **OWASP ZAP**: Web application security testing
- **Vault**: Secret management
- **Falco**: Runtime security monitoring

## Quick Start

1. Run security scan:
   ```bash
   ./security-scan.sh
   ```

2. Check for vulnerabilities:
   ```bash
   trivy image pixelcore/api:latest
   ```

3. Run OWASP ZAP scan:
   ```bash
   docker run -t owasp/zap2docker-stable zap-baseline.py -t https://api.pixelcore.io
   ```

## Security Targets

- Zero high-severity vulnerabilities
- Zero medium-severity vulnerabilities
- All secrets encrypted
- Complete audit logging
- Minimum privilege principle

## Documentation

See [SECURITY_HARDENING.md](../../docs/SECURITY_HARDENING.md) for detailed documentation.
