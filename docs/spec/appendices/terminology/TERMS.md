# Terms

- **artifact**: A file or directory produced by a run, stored under the run directory.
- **run**: A single execution of a DAG that produces a run directory and artifacts.
- **node**: An operation in the DAG with inputs, outputs, and parameters.
- **executor**: A runtime component that executes a node.
- **fingerprint**: A deterministic SHA256 hash of canonical node or graph specs.
- **effect**: A declared side effect (filesystem, network, env) required by a node.
