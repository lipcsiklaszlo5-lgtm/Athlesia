# Negative causal-transfer control

The negative control runs the same production `CrossDomainCausalCore` against a
target graph whose node inventory is compatible but whose causal edge relation is
wrong (`Emits` instead of `Writes`).

```bash
bash tools/verify_cross_domain_causal_negative.sh
```

The run must produce `REFUSED`, install no target model, and preserve a valid
ledger. This is a separate control from the positive causal trace: it demonstrates
that structural transfer is not an unconditional success path.
