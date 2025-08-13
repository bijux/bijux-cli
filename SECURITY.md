# Security Policy

*Last updated: 2025-08-10*

We take security seriously and follow Coordinated Vulnerability Disclosure (CVD). Please report vulnerabilities privately and give us reasonable time to remediate before any public disclosure.

---

## Supported Versions

We patch the **latest minor line** only.

|  Version | Supported |
| -------: | :-------- |
|  `0.1.x` | Yes       |
| `<0.1.0` | No        |

> When `0.2.0` is released, `0.1.x` becomes unsupported. We do not backport beyond the latest minor line.

---

## Reporting a Vulnerability

Please report privately:

* **Preferred:** GitHub **Private Vulnerability Report**
  [https://github.com/bijux/bijux-cli/security/advisories/new](https://github.com/bijux/bijux-cli/security/advisories/new)
* **Fallback:** Email **[mousavi.bijan@gmail.com](mailto:mousavi.bijan@gmail.com)** with subject
  **`[SECURITY] Vulnerability report: bijux-cli`**

### What to include (to speed up triage)

* Affected version(s) and environment (OS, Python, install method)
* Impact and clear reproduction steps
* Minimal Proof-of-Concept (PoC), if available
* Any suggested mitigations/workarounds
* Whether you wish to be credited by name/handle

> Please **do not** include secrets or sensitive data in reports. If you accidentally encounter any, stop testing and report immediately.

---

## Our Process & SLAs (Best Effort)

* **Acknowledgement:** within **48 hours**
* **Initial assessment & provisional CVSS:** within **5 business days**
* **Target fix window** (severity by CVSS v3.x):

  * **Critical:** 7 days
  * **High:** 30 days
  * **Medium:** 90 days
  * **Low:** 180 days

We will publish a **GitHub Security Advisory** with details after a fix is available, and request a **CVE** when appropriate. Reporter credit is given with your consent.

---

## Safe Harbor (Good-Faith Research)

We will not pursue or support legal action for good-faith security research that:

* Avoids privacy violations, data exfiltration, and service interruption.
* Limits testing to accounts/environments you control.
* Respects rate limits; no volumetric DoS or spam.
* Does not exploit beyond what’s necessary to demonstrate impact.
* Stops and reports immediately upon encountering sensitive data.

If you’re unsure whether an activity is in scope of safe harbor, **ask first** via the channels above.

---

## Scope

**In scope**

* This repository’s source code
* Release artifacts we publish
* CLI runtime behavior and default configurations

**Out of scope**

* Social engineering or physical attacks
* Third-party platforms and services (unless our integration directly introduces the issue)
* Volumetric DoS (traffic floods, stress/benchmarking)
* Issues requiring pre-existing privileged local access without a viable escalation path
* Vulnerabilities in third-party **plugins** not maintained in this org

> If you find a dependency vulnerability: report it to the upstream project as well. We’ll track, pin/upgrade, or mitigate on our side as needed.

---

## Proactive Security Practices

* **Dependency auditing:** `pip-audit` with CycloneDX SBOM (`artifacts/sbom.json`)
* **Static analysis:** `bandit` on Python sources
* **Policy gates:** CI blocks on failed security checks; any ignores are reviewed and documented
* **Supply chain:** pinned tooling where feasible; reproducible builds where practical; SBOM generated on release

*(We do not run a bounty program at this time.)*

---

## Contact

* **Private report:** [https://github.com/bijux/bijux-cli/security/advisories/new](https://github.com/bijux/bijux-cli/security/advisories/new)
* **Email:** **[mousavi.bijan@gmail.com](mailto:mousavi.bijan@gmail.com)**
* **Non-security questions:** please open a normal GitHub issue.

Thank you for helping keep Bijux CLI users safe.
