<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please do **not** open a public issue.

### How to Report

1. **Email**: Send details to **<security@xmv.de>**
2. **Subject**: `[SECURITY] talos-api-rs: <brief description>`
3. **Include**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment**: Within 48 hours
- **Initial assessment**: Within 7 days
- **Resolution timeline**: Depends on severity, typically 30-90 days

### Disclosure Policy

- We follow [responsible disclosure](https://en.wikipedia.org/wiki/Responsible_disclosure)
- We will coordinate with you on disclosure timing
- Credit will be given in the security advisory (unless you prefer anonymity)

## Security Best Practices

When using `talos-api-rs`:

1. **Use mTLS for configured nodes** — After initial configuration, always use mTLS
2. **Maintenance Mode is an exception** — Talos nodes in maintenance mode (fresh boot,
   no config) require `insecure: true` because they don't have certificates yet. This
   is the "first machine to connect wins" bootstrap pattern and is a legitimate
   production use case. Secure your network during this phase!
3. **Protect credentials** — Store certificates and keys securely
4. **Rotate certificates** — Follow your organization's certificate rotation policy
5. **Network isolation** — Restrict access to Talos API ports (50000), especially
   critical during maintenance mode bootstrap
6. **Audit logging** — Enable logging to track API calls

## Known Security Considerations

- The library handles sensitive data (certificates, kubeconfig)
- Logging is configurable and can redact sensitive headers
- Circuit breaker prevents accidental DoS of Talos nodes

## Dependencies

We regularly update dependencies via Dependabot. Security updates are prioritized.
