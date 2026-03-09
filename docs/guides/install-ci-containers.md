# Install in CI Containers

Pin the toolchain and install a single channel inside the container image or job.

## Cargo-based image

```bash
cargo install --locked bijux-cli
bijux version
```

## Python-based image

```bash
python -m pip install --upgrade pip
python -m pip install --upgrade bijux-cli
bijux version
```

## CI health checks

```bash
bijux cli paths
bijux cli doctor
```

