use std::{env, fs, process::ExitCode};

use athlesia_integrated_cognitive_agent::byod_causal_transfer::{
    create_byod_proof, verify_byod_proof, ByodProofBundle, ByodTransferRequest,
};

const SOURCE: &str = "episode,action,before.power,before.coolant,before.open,after.power,after.coolant,after.open\n1,press,1,0,0,1,0,1\n2,press,0,1,0,0,1,1\n3,press,1,1,0,1,1,1\n";
const TARGET: &str = "episode,action,after.commit,before.replica,after.quorum,before.commit,after.replica,before.quorum\n1,write,1,0,1,0,0,1\n2,write,1,1,0,0,1,0\n3,write,1,0,0,0,0,0\n";

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  byod_break_test_verify --generate\n  byod_break_test_verify --verify PROOF.json"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [flag] if flag == "--generate" => {
            let bundle = create_byod_proof(ByodTransferRequest::new(SOURCE, TARGET));
            println!(
                "{}",
                serde_json::to_string_pretty(&bundle).expect("serializable proof")
            );
            ExitCode::SUCCESS
        }
        [flag, path] if flag == "--verify" => {
            let raw = match fs::read_to_string(path) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("REFUSED: cannot read {path}: {error}");
                    return ExitCode::from(2);
                }
            };
            let bundle: ByodProofBundle = match serde_json::from_str(&raw) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("REFUSED: invalid proof JSON: {error}");
                    return ExitCode::from(2);
                }
            };
            let result = verify_byod_proof(&bundle);
            if result.verified {
                println!("VERIFIED: ATHLESIA P13 BYOD PROOF {}", result.proof_id);
                ExitCode::SUCCESS
            } else {
                eprintln!("REJECTED: {} ({:?})", result.reason, result.reason_code);
                ExitCode::from(1)
            }
        }
        _ => usage(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_recomputation_accepts_the_exact_bundle() {
        let bundle = create_byod_proof(ByodTransferRequest::new(SOURCE, TARGET));
        assert!(verify_byod_proof(&bundle).verified);
    }

    #[test]
    fn native_recomputation_rejects_changed_input_bytes() {
        let mut bundle = create_byod_proof(ByodTransferRequest::new(SOURCE, TARGET));
        bundle.request.target_csv.push('\n');
        assert!(!verify_byod_proof(&bundle).verified);
    }
}
