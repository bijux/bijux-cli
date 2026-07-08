use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::replay_equivalent;

fn lcg_next(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    *state
}

fn synth_fingerprint(state: &mut u64) -> String {
    let mut buf = String::with_capacity(16);
    for _ in 0..16 {
        let n = (lcg_next(state) & 0x0f) as u8;
        let ch = match n {
            0..=9 => (b'0' + n) as char,
            _ => (b'a' + (n - 10)) as char,
        };
        buf.push(ch);
    }
    buf
}

#[test]
fn replay_equivalence_is_deterministic_for_same_inputs() {
    let a = "fp-stable-a";
    let b = "fp-stable-b";
    let first = replay_equivalent(a, b);
    for _ in 0..100 {
        assert_eq!(first, replay_equivalent(a, b));
    }
}

#[test]
fn replay_equivalence_fuzz_contract_preserves_equality_semantics() {
    let mut state = 0x5EED_CAFE_D00D_u64;
    for _ in 0..2048 {
        let left = synth_fingerprint(&mut state);
        let right = if (lcg_next(&mut state) & 1) == 0 {
            left.clone()
        } else {
            synth_fingerprint(&mut state)
        };
        assert_eq!(replay_equivalent(&left, &right), left == right);
    }
}
