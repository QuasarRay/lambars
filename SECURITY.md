# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of lambars seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

1. **Do NOT** publish vulnerability details in a public issue, pull request, discussion, or commit.
2. If the repository's **Security → Report a vulnerability** control is available, use it to submit the report privately.
3. If private vulnerability reporting is not available, contact the repository owner **QuasarRay** through GitHub first to establish a private disclosure channel; do not include vulnerability details in the initial public contact.
4. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Affected versions/configurations
   - Any suggested fixes (optional)

### What to Expect

- **Acknowledgment**: We will acknowledge receipt within 48 hours
- **Initial Assessment**: We will provide an initial assessment within 7 days
- **Resolution**: We aim to resolve critical vulnerabilities within 30 days

### Disclosure Policy

- We follow responsible disclosure practices
- We will coordinate with you on the disclosure timeline
- We will credit you for the discovery (unless you prefer anonymity)

## Security Measures

### Code Safety

- The crate uses `#![deny(unsafe_code)]` as the default policy.
- A small implementation boundary in the lazy/concurrent-lazy subsystem intentionally contains audited `unsafe` operations for `UnsafeCell` / `MaybeUninit` state management. Those exceptions must remain localized and require dedicated concurrency, Kani, and review evidence.
- The proc-macro and verification crates forbid or deny unsafe code independently.
- Dependencies are regularly updated via Dependabot; advisory/license/source policy is enforced separately in CI.
- CI runs security-focused lints and formal-verification gates.

### Best Practices

When using lambars:

- Keep your dependencies up to date
- Use the latest stable version when possible
- Review the CHANGELOG for security-related updates

## Scope

This security policy applies to:

- The lambars library (`lambars` crate)
- The lambars-derive macro crate (`lambars-derive` crate)

Third-party dependencies are outside the scope of this policy, but we will work with upstream maintainers if a dependency vulnerability affects lambars.
