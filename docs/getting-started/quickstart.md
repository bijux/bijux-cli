# Quickstart

## Purpose
This document gives you a minimal end-to-end run of bijux-cli so you can see the execution model in action. It is designed to validate that the CLI is working and to show you the shape of its output.

## Scope
Quickstart uses a small set of commands and basic output formats. It does not introduce configuration files or plugin behavior.

## First Command
Run `bijux --help` to verify that the CLI responds and that the help system is available.

```bash
bijux --help
```

You should see command categories and a short description of each primary command.

## A Simple Status Check
Next, run a simple command that exercises the normal execution path.

```bash
bijux status
```

This command provides a lightweight health check and shows standard output formatting.

## Structured Output
To see machine-readable output, request JSON format explicitly.

```bash
bijux status --format json
```

The output will be valid JSON. This is the same output you should use in scripts and automation.

## What This Proves
If these commands succeed, you have validated installation, argument parsing, policy resolution, runtime initialization, and output emission.
