# Repeated causal-trace benchmark

Run the bounded causal cycle repeatedly against the same compiled Rust example:

```bash
python3 tools/benchmark_cross_domain_causal_trace.py \
  --release \
  --runs 100 \
  --output artifacts/causal/cross_domain_causal_benchmark_release.json
```

Every iteration independently verifies the nine-stage trace, all seven ledger
records, the previous-hash links, `REVISED` status, epoch 1, and `ledger_valid=true`.
The report then computes min, p50, p95, and max from the runner's measured interval.

The interval is the bounded example's internal `Instant` measurement, including its
trace emission. It is not a whole-process latency, a universal upper bound, or a claim about arbitrary input
sizes. Publish it with the commit, profile, hardware, run count, and fixture scope.
The benchmark deliberately fails if any iteration produces a different trace or an
invalid ledger.
