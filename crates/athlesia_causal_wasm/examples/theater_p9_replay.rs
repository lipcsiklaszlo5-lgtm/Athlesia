//! A native reproduction of exactly the P9 receiver used by the browser.
//! No browser, signing secrets, JavaScript success flags or model API needed.
use athlesia_causal_wasm::theater::{decode_key, TheaterOrigin, TheaterReceiver};
use serde_json::{json, Value};

fn replay(document: &Value) -> Result<(), String> {
    if document["schema"] != "athlesia/p9-reproduction/1" {
        return Err("Unknown proof schema".into());
    }
    let seed = document["seed"]
        .as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .ok_or("Bad seed")?;
    let key = |field: &str| decode_key(document[field].as_str().ok_or("Missing public key")?);
    let mut receiver = TheaterReceiver::new(seed, &key("public_a")?, &key("public_b")?)?;
    let transcript = document["transcript"].as_array().ok_or("No transcript")?;
    if transcript.is_empty() || transcript.len() > 32 {
        return Err("Transcript budget".into());
    }
    let mut completed = false;
    let mut blocked = false;
    let mut replay_rejected = false;
    for (index, step) in transcript.iter().enumerate() {
        let request = step["request"]
            .as_str()
            .ok_or("Missing original Rust request bytes")?;
        let result = receiver.step(request);
        if let Some(error) = step["error"].as_str() {
            if result.as_ref().err().map(String::as_str) != Some(error) {
                return Err(format!("Different rejection at step {index}"));
            }
            let decoded: Value = serde_json::from_str(request).map_err(|e| e.to_string())?;
            if completed && decoded["op"] == "dispatch" && decoded["host_allows"] == true {
                replay_rejected = true;
            }
        } else {
            let response = result?;
            if response != step["response"] {
                return Err(format!("Different Rust result at step {index}"));
            }
            blocked |= response["status"] == "ABSTAIN";
            completed |= response["status"] == "CONFIRMED";
        }
    }
    if !blocked || !completed || !replay_rejected {
        return Err("Incomplete P9 proof: need block, confirmation, and rejected replay".into());
    }
    Ok(())
}

fn generate(seed: u32) -> Result<Value, String> {
    // Publicly documented native test keys, used only for reproducible local trials.
    // The browser instead generates fresh secrets in separate origin Workers.
    let mut a = TheaterOrigin::new(seed, 7001, &[11; 32])?;
    let b = TheaterOrigin::new(seed, 7002, &[22; 32])?;
    let mut receiver = TheaterReceiver::new(
        seed,
        &decode_key(&a.public_key())?,
        &decode_key(&b.public_key())?,
    )?;
    let initial = a.bundle(1)?;
    let fault: Value = serde_json::from_str(&a.sabotage()?).map_err(|e| e.to_string())?;
    let restored = a.restore()?;
    let mut transcript = Vec::new();
    for request in [
        json!({"op":"learn", "a":initial, "b":b.bundle(1)?}),
        json!({"op":"sabotage", "revocation":fault["revocation"]}),
        json!({"op":"remap", "a":restored}),
        json!({"op":"alien", "a":a.bundle(3)?, "b":b.bundle(1)?}),
        json!({"op":"dispatch", "host_allows":false}),
        json!({"op":"dispatch", "host_allows":true}),
        json!({"op":"settle", "observed":true}),
        json!({"op":"dispatch", "host_allows":true}),
    ] {
        let request = request.to_string();
        transcript.push(match receiver.step(&request) {
            Ok(response) => json!({"request":request, "response":response}),
            Err(error) => json!({"request":request, "error":error}),
        });
    }
    let proof = json!({"schema":"athlesia/p9-reproduction/1", "seed":seed,
        "public_a":a.public_key(), "public_b":b.public_key(), "transcript":transcript,
        "scope":"native P9 receiver replay; does not independently observe browser IndexedDB or physical hardware"});
    replay(&proof)?;
    Ok(proof)
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--verify") => {
            let path = args.next().ok_or("Missing proof filename")?;
            if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 2_000_000 {
                return Err("Proof file budget".into());
            }
            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
            if bytes.len() > 2_000_000 {
                return Err("Proof file budget".into());
            }
            let proof = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            replay(&proof)?;
            println!("VERIFIED: Ed25519 / CKS import / stale P9 lease / fresh admission / outcome / rejected replay");
        }
        seed => {
            let seed = seed
                .unwrap_or("42")
                .parse::<u32>()
                .map_err(|_| "Expected a u32 seed")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&generate(seed)?).map_err(|e| e.to_string())?
            );
        }
    }
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("NOT VERIFIED: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_native_p9_runs_reproduce_with_no_signing_secret_in_the_proof() {
        for seed in [0, 1, 42, 77, u32::MAX] {
            let document = generate(seed).unwrap();
            replay(&document).unwrap();
            assert!(!document.to_string().contains("secret"));
        }
    }
    #[test]
    fn changing_a_rust_verdict_in_the_download_is_rejected() {
        let mut document = generate(42).unwrap();
        document["transcript"][1]["response"]["dispatch_issued"] = json!(true);
        assert!(replay(&document).is_err());
    }
    #[test]
    fn a_signed_packet_cannot_be_changed_without_authentication_failure() {
        let seed = 7;
        let a = TheaterOrigin::new(seed, 7001, &[11; 32]).unwrap();
        let b = TheaterOrigin::new(seed, 7002, &[22; 32]).unwrap();
        let mut receiver = TheaterReceiver::new(
            seed,
            &decode_key(&a.public_key()).unwrap(),
            &decode_key(&b.public_key()).unwrap(),
        )
        .unwrap();
        let packet = a
            .bundle(1)
            .unwrap()
            .replace("\"origin_sequence\":1", "\"origin_sequence\":2");
        let error = receiver
            .step(&json!({"op":"learn","a":packet,"b":b.bundle(1).unwrap()}).to_string())
            .unwrap_err();
        assert!(
            error.contains("Authentication") || error.contains("Signature"),
            "{error}"
        );
    }
    #[test]
    fn wrong_phase_and_false_database_observation_cannot_report_success() {
        let a = TheaterOrigin::new(9, 7001, &[11; 32]).unwrap();
        let b = TheaterOrigin::new(9, 7002, &[22; 32]).unwrap();
        let mut receiver = TheaterReceiver::new(
            9,
            &decode_key(&a.public_key()).unwrap(),
            &decode_key(&b.public_key()).unwrap(),
        )
        .unwrap();
        assert!(receiver
            .step(r#"{"op":"dispatch","host_allows":true}"#)
            .is_err());
        let document = generate(9).unwrap();
        for step in document["transcript"].as_array().unwrap().iter().take(6) {
            let _ = receiver.step(step["request"].as_str().unwrap());
        }
        assert!(receiver
            .step(r#"{"op":"settle","observed":false}"#)
            .unwrap_err()
            .contains("contradicted"));
        assert!(receiver.step(r#"{"op":"settle","observed":true}"#).is_err());
    }

    #[test]
    fn missing_host_outcome_is_terminal_indeterminate_and_cannot_be_retried() {
        let proof = generate(101).unwrap();
        let mut receiver = TheaterReceiver::new(
            101,
            &decode_key(proof["public_a"].as_str().unwrap()).unwrap(),
            &decode_key(proof["public_b"].as_str().unwrap()).unwrap(),
        )
        .unwrap();
        for step in proof["transcript"].as_array().unwrap().iter().take(6) {
            let _ = receiver.step(step["request"].as_str().unwrap());
        }
        let outcome = receiver.step(r#"{"op":"missing"}"#).unwrap();
        assert_eq!(outcome["status"], "ABSTAIN");
        assert!(receiver
            .step(r#"{"op":"dispatch","host_allows":true}"#)
            .is_err());
        assert!(receiver.step(r#"{"op":"settle","observed":true}"#).is_err());
    }
}
