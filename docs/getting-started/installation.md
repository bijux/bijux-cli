# Installation

## Purpose
This document explains how to install bijux-cli in a way that is reproducible and easy to validate. It exists so you can reach a known-good install quickly and avoid environment-specific confusion.

## Scope
It covers supported installation methods and the minimal verification step. It does not cover upgrading, uninstallation, or system package managers.

## Recommended Installation
Use pip or hatch to install the package into an isolated environment. This keeps dependency resolution predictable and makes it easier to reproduce issues.

```bash
python -m pip install bijux-cli
```

## Verification
After installation, confirm the binary is on your path and responds correctly.

```bash
bijux --version
```

If the command prints a version and exits with code 0, the installation is valid.

## Alternatives
System package managers may be convenient, but they can introduce version skew. We recommend pip or hatch because they align with how bijux-cli is published and tested.
