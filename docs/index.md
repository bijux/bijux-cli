# Bijux CLI Documentation

## Purpose
This documentation set explains how to use and extend bijux-cli with confidence. It exists to give experienced engineers new to bijux-cli a clear mental model, precise guarantees, and concrete guidance without forcing them to read the code first.

## Scope
These docs cover the CLI, its execution model, configuration and precedence rules, plugin lifecycle expectations, and the public contract between the CLI and the API. They do not attempt to mirror internal implementation details beyond what is needed to explain behavior and guarantees.

## Audience
This material is written for engineers who are comfortable with CLI tools and configuration-driven systems, but who are new to bijux-cli. It assumes you can read command-line examples and understand basic concepts like environment variables and configuration files.

## How to Navigate
Start with Getting Started if you want to install and run bijux-cli quickly. Use Concepts to understand the execution model and guarantees. Guides provide task-oriented instructions for common workflows, and Reference provides authoritative tables and lists for commands and configuration.

## Sections
- Getting Started: a concise path from installation to your first successful commands.
- Concepts: the guarantees, invariants, and mental models that govern behavior.
- Guides: practical, step-by-step workflows for real usage.
- Reference: canonical lists of commands, config schema, and exit codes.
- Constitution: normative compatibility contracts for command identity, flags, output, errors, and deprecation.
- Architecture: the documented decision rules and execution walk-through.
