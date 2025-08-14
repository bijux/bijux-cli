# Security Policy
<a id="top"></a>

*Last updated: 2025-08-10*

We follow **Coordinated Vulnerability Disclosure (CVD)**. Please report issues privately and allow time for remediation before public disclosure.

[Back to top](#top)

---

## Table of Contents

- [Supported Versions](#supported-versions)
- [Reporting a Vulnerability](#reporting-a-vulnerability)
- [Our Process & SLAs](#our-process--slas)
- [Safe Harbor (Good-Faith Research)](#safe-harbor-good-faith-research)
- [Scope](#scope)
- [Proactive Security Practices](#proactive-security-practices)
- [Contact](#contact)

[Back to top](#top)

---

<a id="supported-versions"></a>

## Supported Versions

We patch the **latest minor line** only.

|  Version | Supported |
| -------: | :-------- |
|  `0.1.x` | Yes       |
| `<0.1.0` | No        |

When `0.2.0` is released, support for `0.1.x` ends. We do **not** backport beyond the most recent minor line.

[Back to top](#top)

---

<a id="reporting-a-vulnerability"></a>

## Reporting a Vulnerability

Please report privately via one of the following channels:

- **Preferred:** GitHub **Private Vulnerability Report**  
  https://github.com/bijux/bijux-cli/security/advisories/new
- **Fallback:** Email **[mousavi.bijan@gmail.com](mailto:mousavi.bijan@gmail.com)** with subject  
  **`[SECURITY] Vulnerability report: bijux-cli`**

### What to include (to speed up triage)

- Affected version(s), OS, Python version, and install method  
- Clear impact statement and **reproduction steps**  
- Minimal **PoC** if possible  
- Suggested mitigations/workarounds (if any)  
- Whether you’d like **credit** (name/handle)

> Please **do not** include secrets or production data. If you encounter sensitive information, stop testing and report immediately.

[Back to top](#top)

---

<a id="our-process--slas"></a>

## Our Process & SLAs

Best-effort targets based on **CVSS v3.x** severity:

- **Acknowledgement:** within **48 hours**  
- **Initial assessment & provisional CVSS:** within **5 business days**  
- **Target fix windows:**
  - **Critical:** 7 days
  - **High:** 30 days
  - **Medium:** 90 days
  - **Low:** 180 days

We publish a **GitHub Security Advisory** once a fix is available and request a **CVE** when appropriate. Reporter credit is given with your consent.

[Back to top](#top)

---

<a id="safe-harbor-good-faith-research"></a>

## Safe Harbor (Good-Faith Research)

We won’t pursue or support legal action for good-faith testing that:

- Avoids privacy violations, data exfiltration, and service interruption  
- Is limited to accounts/environments you control  
- Respects rate limits (no volumetric DoS/spam)  
- Does not escalate or persist beyond what’s necessary to demonstrate impact  
- Stops and reports immediately upon encountering sensitive data

If you’re unsure whether an activity is in scope, **ask first** via the channels above.

[Back to top](#top)

---

<a id="scope"></a>

## Scope

**In scope**

- This repository’s source code  
- Release artifacts we publish  
- CLI runtime behavior and default configurations

**Out of scope**

- Social engineering or physical attacks  
- Third-party platforms/services (unless our integration directly introduces the issue)  
- Volumetric DoS (traffic floods, stress/benchmarking)  
- Issues requiring pre-existing privileged local access without a plausible escalation path  
- Vulnerabilities in third-party **plugins** not maintained by this org

> For dependency vulnerabilities, please also notify the **upstream** project. We will track, pin/upgrade, or mitigate downstream as needed.

[Back to top](#top)

---

<a id="proactive-security-practices"></a>

## Proactive Security Practices

- **Dependency auditing:** `pip-audit`; SBOM via CycloneDX (`artifacts/sbom.json`)  
- **Static analysis:** `bandit` on Python sources  
- **Policy gates:** CI blocks on failed security checks; any ignores are reviewed and documented  
- **Supply chain:** pinned tooling where feasible; reproducible builds where practical; SBOM generated on release

*(No public bounty program at this time.)*

[Back to top](#top)

---

<a id="contact"></a>

## Contact

- **Private report:** https://github.com/bijux/bijux-cli/security/advisories/new  
- **Email:** **[mousavi.bijan@gmail.com](mailto:mousavi.bijan@gmail.com)**  
- **Non-security questions:** open a normal GitHub issue

Thank you for helping keep Bijux CLI users safe.

[Back to top](#top)
