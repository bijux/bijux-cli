#![forbid(unsafe_code)]
//! Randomized state-corruption harness coverage for config/registry/history/memory/install metadata.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use bijux_cli_core as _;
use bijux_cli_install as _;
use bijux_cli_python as _;
use bijux_cli_repl as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

#[derive(Clone, Copy, Debug)]
enum Domain {
    Config,
    PluginRegistry,
    History,
    Memory,
    InstallMetadata,
}

#[derive(Clone, Copy, Debug)]
enum Mutator {
    Truncation,
    ByteFlip,
    LineDeletion,
    LineDuplication,
    KeyDuplication,
    WrongTypeInjection,
    MissingFieldDeletion,
    ExtraFieldInsertion,
    InvalidEncodingInjection,
    TemporaryFileLeftover,
    PartialWriteLeftover,
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }

    fn bounded(&mut self, upper: usize) -> usize {
        if upper == 0 {
            0
        } else {
            (self.next_u64() as usize) % upper
        }
    }
}

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "bijux-state-corruption-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("mkdir temp root");
    path
}

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("command should execute")
}

fn seed_for(domain: Domain) -> Vec<u8> {
    match domain {
        Domain::Config => b"BIJUXCLI_ALPHA=one\nBIJUXCLI_BETA=two\n".to_vec(),
        Domain::PluginRegistry => {
            b"{\"plugins\":[{\"name\":\"sample\",\"path\":\"/tmp/sample\",\"enabled\":true}]}"
                .to_vec()
        }
        Domain::History => b"status\ndoctor\n".to_vec(),
        Domain::Memory => b"{\"alpha\":{\"value\":\"1\"},\"beta\":{\"value\":\"2\"}}".to_vec(),
        Domain::InstallMetadata => {
            b"{\"install\":{\"channel\":\"cargo\",\"binary\":\"bijux-rs\",\"version\":\"0.0.0\"}}"
                .to_vec()
        }
    }
}

fn apply_mutator(path: &Path, mutator: Mutator, rng: &mut Lcg, seed: &[u8]) -> io::Result<()> {
    match mutator {
        Mutator::Truncation => {
            let keep = rng.bounded(seed.len().max(1));
            fs::write(path, &seed[..keep])
        }
        Mutator::ByteFlip => {
            let mut bytes = seed.to_vec();
            if bytes.is_empty() {
                bytes.push(0);
            }
            let idx = rng.bounded(bytes.len());
            bytes[idx] ^= 0xFF;
            fs::write(path, bytes)
        }
        Mutator::LineDeletion => {
            let text = String::from_utf8_lossy(seed);
            let mut lines: Vec<&str> = text.lines().collect();
            if !lines.is_empty() {
                lines.remove(rng.bounded(lines.len()));
            }
            fs::write(path, lines.join("\n"))
        }
        Mutator::LineDuplication => {
            let text = String::from_utf8_lossy(seed);
            let mut lines: Vec<String> = text.lines().map(ToOwned::to_owned).collect();
            if !lines.is_empty() {
                let idx = rng.bounded(lines.len());
                let dup = lines[idx].clone();
                lines.insert(idx, dup);
            }
            fs::write(path, lines.join("\n"))
        }
        Mutator::KeyDuplication => {
            let text = String::from_utf8_lossy(seed);
            let mut lines: Vec<String> = text.lines().map(ToOwned::to_owned).collect();
            if let Some(first) = lines.first().cloned() {
                lines.push(first);
            }
            fs::write(path, lines.join("\n"))
        }
        Mutator::WrongTypeInjection => {
            let text = String::from_utf8_lossy(seed);
            let mutated = if text.trim_start().starts_with('{') {
                "{\"plugins\":\"wrong-type\"}".to_string()
            } else {
                "BIJUXCLI_ALPHA=[1,2,3]".to_string()
            };
            fs::write(path, mutated)
        }
        Mutator::MissingFieldDeletion => {
            let text = String::from_utf8_lossy(seed);
            let mutated =
                text.replace("\"path\":\"/tmp/sample\",", "").replace("BIJUXCLI_BETA=two\n", "");
            fs::write(path, mutated)
        }
        Mutator::ExtraFieldInsertion => {
            let text = String::from_utf8_lossy(seed);
            let mutated = if text.trim_start().starts_with('{') {
                format!("{}\n{}", text, "{\"unexpected\":true}")
            } else {
                format!("{}\nBIJUXCLI_EXTRA_FIELD=unexpected", text)
            };
            fs::write(path, mutated)
        }
        Mutator::InvalidEncodingInjection => {
            let mut bytes = seed.to_vec();
            bytes.extend_from_slice(&[0xFF, 0xFE, 0x00]);
            fs::write(path, bytes)
        }
        Mutator::TemporaryFileLeftover => {
            fs::write(path, seed)?;
            fs::write(path.with_extension("tmp"), b"stale temp")
        }
        Mutator::PartialWriteLeftover => {
            fs::write(path, seed)?;
            let mut partial = seed.to_vec();
            let keep = rng.bounded(partial.len().max(1));
            partial.truncate(keep);
            fs::write(path.with_extension("partial"), partial)
        }
    }
}

fn write_domain_state(root: &Path, domain: Domain, mutator: Mutator, rng: &mut Lcg) -> PathBuf {
    let path = match domain {
        Domain::Config => root.join("config.env"),
        Domain::PluginRegistry => root.join("plugins").join("registry.json"),
        Domain::History => root.join("history.log"),
        Domain::Memory => root.join("home").join(".bijux").join(".memory.json"),
        Domain::InstallMetadata => root.join("home").join(".bijux").join("install-state.json"),
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir state parent");
    }

    let seed = seed_for(domain);
    apply_mutator(&path, mutator, rng, &seed).expect("mutator should write state");
    path
}

fn exercise_domain(root: &Path, domain: Domain, target: &Path) -> Output {
    let home = root.join("home");
    let plugins = root.join("plugins");
    let history = root.join("history.log");

    fs::create_dir_all(home.join(".bijux")).expect("mkdir home/.bijux");
    fs::create_dir_all(&plugins).expect("mkdir plugins");

    let mut envs = vec![
        ("HOME", home.display().to_string()),
        ("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string()),
        ("BIJUXCLI_HISTORY_FILE", history.display().to_string()),
    ];

    match domain {
        Domain::Config => run_with_env(
            &[
                "dev",
                "cli",
                "state-doctor",
                "--format",
                "json",
                "--no-pretty",
                "--config-path",
                target.to_str().expect("utf-8 config path"),
            ],
            &envs,
        ),
        Domain::PluginRegistry => {
            run_with_env(&["cli", "plugins", "doctor", "--format", "json", "--no-pretty"], &envs)
        }
        Domain::History => run_with_env(&["history", "--format", "json", "--no-pretty"], &envs),
        Domain::Memory => {
            run_with_env(&["memory", "list", "--format", "json", "--no-pretty"], &envs)
        }
        Domain::InstallMetadata => {
            envs.push(("BIJUXCLI_INSTALL_STATE_FILE", target.display().to_string()));
            run_with_env(
                &["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"],
                &envs,
            )
        }
    }
}

#[test]
fn randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains() {
    let mut rng = Lcg::new(0xC0FFEE1234u64);
    let root = temp_dir("campaign");

    let domains = [
        Domain::Config,
        Domain::PluginRegistry,
        Domain::History,
        Domain::Memory,
        Domain::InstallMetadata,
    ];
    let mutators = [
        Mutator::Truncation,
        Mutator::ByteFlip,
        Mutator::LineDeletion,
        Mutator::LineDuplication,
        Mutator::KeyDuplication,
        Mutator::WrongTypeInjection,
        Mutator::MissingFieldDeletion,
        Mutator::ExtraFieldInsertion,
        Mutator::InvalidEncodingInjection,
        Mutator::TemporaryFileLeftover,
        Mutator::PartialWriteLeftover,
    ];

    let mut exercised = 0usize;
    for domain in domains {
        for mutator in mutators {
            let domain_root = root.join(format!("{:?}-{:?}", domain, mutator));
            fs::create_dir_all(&domain_root).expect("mkdir case root");
            let target = write_domain_state(&domain_root, domain, mutator, &mut rng);
            let out = exercise_domain(&domain_root, domain, &target);
            assert!(
                matches!(out.status.code(), Some(0) | Some(1)),
                "domain={domain:?} mutator={mutator:?} crashed with status={:?}",
                out.status.code()
            );
            exercised += 1;
        }
    }

    assert_eq!(exercised, 55);
}
