//! Deterministic continuous-input abstraction for the causal core.
//!
//! The boundary accepts fixed-point scalar samples, applies a bounded
//! quantizer with optional hysteresis, and emits a compact symbolic state.
//! A small binary hypervector is bundled from the same categorical atoms for
//! tolerant similarity checks. No learned weights, floating-point state, or
//! unbounded feature expansion crosses this boundary.

use std::collections::{BTreeMap, BTreeSet};

use super::cross_domain_causal_core::{Digest, Literal, State, TokenId};

pub const MAX_ABSTRACTION_CHANNELS: usize = 64;
pub const MAX_HDC_WORDS: usize = 32;
pub const MAX_BINS: u8 = 16;
const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;
const CONFIG_DOMAIN: u64 = 0x4154_4c45_5349_4341;
const ATOM_DOMAIN: u64 = 0x4154_4c45_5349_4154;

fn mix(mut state: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        state ^= u64::from(byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut result = value;
    result = (result ^ (result >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    result = (result ^ (result >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    result ^ (result >> 31)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbstractionError {
    EmptySchema,
    EmptyFrame,
    DuplicateRole(TokenId),
    UnknownRole(TokenId),
    MissingRole(TokenId),
    ChannelBudgetExceeded { observed: usize, limit: usize },
    InvalidRange { role: TokenId },
    InvalidBinCount { bins: u8 },
    InvalidHysteresis,
    InvalidHdcDimension { words: usize },
    StateBudgetExceeded { required: usize, limit: usize },
    SymbolicRoleCollision { role: TokenId },
    NonFiniteInput,
    NumericOverflow,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FixedScalar {
    role: TokenId,
    micros: i64,
}

impl FixedScalar {
    pub fn from_micros(role: TokenId, micros: i64) -> Self {
        Self { role, micros }
    }

    pub fn from_f64(role: TokenId, value: f64) -> Result<Self, AbstractionError> {
        if !value.is_finite() {
            return Err(AbstractionError::NonFiniteInput);
        }
        let scaled = value * 1_000_000.0;
        if scaled < i64::MIN as f64 || scaled > i64::MAX as f64 {
            return Err(AbstractionError::NumericOverflow);
        }
        Ok(Self {
            role,
            micros: scaled.round() as i64,
        })
    }

    pub fn role(self) -> TokenId { self.role }
    pub fn micros(self) -> i64 { self.micros }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ChannelRange {
    role: TokenId,
    min_micros: i64,
    max_micros: i64,
}

impl ChannelRange {
    pub fn new(
        role: TokenId,
        min_micros: i64,
        max_micros: i64,
    ) -> Result<Self, AbstractionError> {
        if min_micros >= max_micros {
            return Err(AbstractionError::InvalidRange { role });
        }
        Ok(Self {
            role,
            min_micros,
            max_micros,
        })
    }

    pub fn role(self) -> TokenId { self.role }
    pub fn min_micros(self) -> i64 { self.min_micros }
    pub fn max_micros(self) -> i64 { self.max_micros }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuousFrame {
    samples: Vec<FixedScalar>,
}

impl ContinuousFrame {
    pub fn new(mut samples: Vec<FixedScalar>) -> Result<Self, AbstractionError> {
        if samples.is_empty() {
            return Err(AbstractionError::EmptyFrame);
        }
        if samples.len() > MAX_ABSTRACTION_CHANNELS {
            return Err(AbstractionError::ChannelBudgetExceeded {
                observed: samples.len(),
                limit: MAX_ABSTRACTION_CHANNELS,
            });
        }
        samples.sort();
        for pair in samples.windows(2) {
            if pair[0].role == pair[1].role {
                return Err(AbstractionError::DuplicateRole(pair[0].role));
            }
        }
        Ok(Self { samples })
    }

    pub fn samples(&self) -> &[FixedScalar] { &self.samples }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeterministicAbstractionConfig {
    ranges: Vec<ChannelRange>,
    bins: u8,
    hysteresis_micros: i64,
    hdc_words: usize,
    max_state_literals: usize,
    seed: u64,
    schema_digest: Digest,
}

impl DeterministicAbstractionConfig {
    pub fn new(
        mut ranges: Vec<ChannelRange>,
        bins: u8,
        hysteresis_micros: i64,
        hdc_words: usize,
        max_state_literals: usize,
        seed: u64,
    ) -> Result<Self, AbstractionError> {
        if ranges.is_empty() {
            return Err(AbstractionError::EmptySchema);
        }
        if ranges.len() > MAX_ABSTRACTION_CHANNELS {
            return Err(AbstractionError::ChannelBudgetExceeded {
                observed: ranges.len(),
                limit: MAX_ABSTRACTION_CHANNELS,
            });
        }
        if !(2..=MAX_BINS).contains(&bins) {
            return Err(AbstractionError::InvalidBinCount { bins });
        }
        if hysteresis_micros < 0 {
            return Err(AbstractionError::InvalidHysteresis);
        }
        if !(1..=MAX_HDC_WORDS).contains(&hdc_words) {
            return Err(AbstractionError::InvalidHdcDimension { words: hdc_words });
        }
        ranges.sort();
        for pair in ranges.windows(2) {
            if pair[0].role == pair[1].role {
                return Err(AbstractionError::DuplicateRole(pair[0].role));
            }
        }
        let required = ranges
            .len()
            .checked_mul(usize::from(bins))
            .ok_or(AbstractionError::NumericOverflow)?;
        if max_state_literals < required {
            return Err(AbstractionError::StateBudgetExceeded {
                required,
                limit: max_state_literals,
            });
        }
        let schema_digest = config_digest(&ranges, bins, hysteresis_micros, hdc_words, seed);
        Ok(Self {
            ranges,
            bins,
            hysteresis_micros,
            hdc_words,
            max_state_literals,
            seed,
            schema_digest,
        })
    }

    pub fn ranges(&self) -> &[ChannelRange] { &self.ranges }
    pub fn bins(&self) -> u8 { self.bins }
    pub fn hysteresis_micros(&self) -> i64 { self.hysteresis_micros }
    pub fn hdc_words(&self) -> usize { self.hdc_words }
    pub fn max_state_literals(&self) -> usize { self.max_state_literals }
    pub fn seed(&self) -> u64 { self.seed }
    pub fn schema_digest(&self) -> Digest { self.schema_digest }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SymbolicAtom {
    pub role: TokenId,
    pub bin: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolicObservation {
    schema_digest: Digest,
    abstraction_digest: Digest,
    atoms: Vec<SymbolicAtom>,
    state: State,
    hypervector: Vec<u64>,
}

impl SymbolicObservation {
    pub fn schema_digest(&self) -> Digest { self.schema_digest }
    pub fn abstraction_digest(&self) -> Digest { self.abstraction_digest }
    pub fn atoms(&self) -> &[SymbolicAtom] { &self.atoms }
    pub fn state(&self) -> &State { &self.state }
    pub fn hypervector(&self) -> &[u64] { &self.hypervector }

    pub fn hamming_distance(&self, other: &Self) -> Option<u32> {
        (self.hypervector.len() == other.hypervector.len()).then(|| {
            self.hypervector
                .iter()
                .zip(&other.hypervector)
                .map(|(left, right)| (left ^ right).count_ones())
                .sum()
        })
    }
}

pub trait SymbolicAbstraction {
    fn abstract_frame(
        &mut self,
        frame: &ContinuousFrame,
    ) -> Result<SymbolicObservation, AbstractionError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeterministicAbstraction {
    config: DeterministicAbstractionConfig,
    previous_bins: BTreeMap<TokenId, u8>,
}

impl DeterministicAbstraction {
    pub fn new(config: DeterministicAbstractionConfig) -> Self {
        Self {
            config,
            previous_bins: BTreeMap::new(),
        }
    }

    pub fn config(&self) -> &DeterministicAbstractionConfig { &self.config }

    pub fn reset_hysteresis(&mut self) { self.previous_bins.clear(); }
}

impl SymbolicAbstraction for DeterministicAbstraction {
    fn abstract_frame(
        &mut self,
        frame: &ContinuousFrame,
    ) -> Result<SymbolicObservation, AbstractionError> {
        if frame.samples().len() > self.config.ranges.len() {
            return Err(AbstractionError::ChannelBudgetExceeded {
                observed: frame.samples().len(),
                limit: self.config.ranges.len(),
            });
        }
        let samples: BTreeMap<TokenId, i64> = frame
            .samples()
            .iter()
            .map(|sample| (sample.role(), sample.micros()))
            .collect();
        let ranges: BTreeMap<TokenId, ChannelRange> = self
            .config
            .ranges
            .iter()
            .copied()
            .map(|range| (range.role(), range))
            .collect();
        if let Some(role) = samples.keys().find(|role| !ranges.contains_key(role)) {
            return Err(AbstractionError::UnknownRole(*role));
        }
        if let Some(role) = ranges.keys().find(|role| !samples.contains_key(role)) {
            return Err(AbstractionError::MissingRole(*role));
        }

        let mut atoms = Vec::with_capacity(self.config.ranges.len());
        for range in &self.config.ranges {
            let value = samples[&range.role()];
            let raw_bin = quantize(value, *range, self.config.bins);
            let selected_bin = self
                .previous_bins
                .get(&range.role())
                .copied()
                .map(|previous| {
                    hysteretic_bin(
                        value,
                        *range,
                        self.config.bins,
                        self.config.hysteresis_micros,
                        previous,
                        raw_bin,
                    )
                })
                .unwrap_or(raw_bin);
            atoms.push(SymbolicAtom {
                role: range.role(),
                bin: selected_bin,
            });
        }

        let mut state_literals = Vec::with_capacity(atoms.len() * usize::from(self.config.bins));
        let mut symbolic_roles = BTreeSet::new();
        for atom in &atoms {
            for bin in 0..self.config.bins {
                let role = atom_token(self.config.seed, atom.role, bin);
                if !symbolic_roles.insert(role) {
                    return Err(AbstractionError::SymbolicRoleCollision { role });
                }
                state_literals.push(Literal::new(role, bin == atom.bin));
            }
        }
        if state_literals.len() > self.config.max_state_literals {
            return Err(AbstractionError::StateBudgetExceeded {
                required: state_literals.len(),
                limit: self.config.max_state_literals,
            });
        }

        let hypervector = bundle(
            atoms.iter().copied(),
            self.config.hdc_words,
            self.config.seed,
        );
        let state = State::new(state_literals);
        let abstraction_digest = observation_digest(
            self.config.schema_digest,
            &atoms,
            &hypervector,
            state.digest(),
        );
        self.previous_bins = atoms.iter().map(|atom| (atom.role, atom.bin)).collect();
        Ok(SymbolicObservation {
            schema_digest: self.config.schema_digest,
            abstraction_digest,
            atoms,
            state,
            hypervector,
        })
    }
}

fn config_digest(
    ranges: &[ChannelRange],
    bins: u8,
    hysteresis_micros: i64,
    hdc_words: usize,
    seed: u64,
) -> Digest {
    let mut digest = mix(FNV_OFFSET, CONFIG_DOMAIN);
    digest = mix(digest, u64::from(bins));
    digest = mix(digest, hysteresis_micros as u64);
    digest = mix(digest, hdc_words as u64);
    digest = mix(digest, seed);
    digest = mix(digest, ranges.len() as u64);
    for range in ranges {
        digest = mix(digest, range.role());
        digest = mix(digest, range.min_micros() as u64);
        digest = mix(digest, range.max_micros() as u64);
    }
    digest
}

fn atom_token(seed: u64, role: TokenId, bin: u8) -> TokenId {
    let mut digest = mix(FNV_OFFSET, ATOM_DOMAIN);
    digest = mix(digest, seed);
    digest = mix(digest, role);
    mix(digest, u64::from(bin)).max(1)
}

fn quantize(value: i64, range: ChannelRange, bins: u8) -> u8 {
    let value = value.clamp(range.min_micros(), range.max_micros());
    let width = i128::from(range.max_micros()) - i128::from(range.min_micros());
    let offset = i128::from(value) - i128::from(range.min_micros());
    let raw = (offset * i128::from(bins)) / width;
    raw.min(i128::from(bins) - 1) as u8
}

fn hysteretic_bin(
    value: i64,
    range: ChannelRange,
    bins: u8,
    hysteresis_micros: i64,
    previous: u8,
    raw: u8,
) -> u8 {
    if raw == previous
        || hysteresis_micros == 0
        || previous >= bins
        || raw >= bins
        || previous.abs_diff(raw) != 1
    {
        return raw;
    }
    let boundary_bin = previous.min(raw).saturating_add(1);
    let width = i128::from(range.max_micros()) - i128::from(range.min_micros());
    let boundary = i128::from(range.min_micros())
        + (width * i128::from(boundary_bin)) / i128::from(bins);
    let distance = (i128::from(value) - boundary).unsigned_abs();
    if distance <= hysteresis_micros as u128 {
        previous
    } else {
        raw
    }
}

fn item_vector(seed: u64, atom: SymbolicAtom, words: usize) -> Vec<u64> {
    let basis = seed
        ^ atom.role.rotate_left(17)
        ^ u64::from(atom.bin).rotate_left(41);
    (0..words)
        .map(|index| splitmix64(basis ^ (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)))
        .collect()
}

fn bundle(
    atoms: impl Iterator<Item = SymbolicAtom>,
    words: usize,
    seed: u64,
) -> Vec<u64> {
    let atoms = atoms.collect::<Vec<_>>();
    let mut ones = vec![0u16; words * 64];
    for atom in atoms.iter().copied() {
        for (word_index, word) in item_vector(seed, atom, words).into_iter().enumerate() {
            for bit in 0..64 {
                ones[word_index * 64 + bit] += u16::from((word >> bit) & 1 == 1);
            }
        }
    }
    let mut output = vec![0u64; words];
    for (index, count) in ones.into_iter().enumerate() {
        if count.saturating_mul(2) >= atoms.len() as u16 {
            output[index / 64] |= 1u64 << (index % 64);
        }
    }
    output
}

fn observation_digest(
    schema_digest: Digest,
    atoms: &[SymbolicAtom],
    hypervector: &[u64],
    state_digest: Digest,
) -> Digest {
    let mut digest = mix(schema_digest, state_digest);
    digest = mix(digest, atoms.len() as u64);
    for atom in atoms {
        digest = mix(digest, atom.role);
        digest = mix(digest, u64::from(atom.bin));
    }
    for word in hypervector {
        digest = mix(digest, *word);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(hysteresis_micros: i64) -> DeterministicAbstractionConfig {
        DeterministicAbstractionConfig::new(
            vec![
                ChannelRange::new(10, 0, 1_000_000).unwrap(),
                ChannelRange::new(20, -1_000_000, 1_000_000).unwrap(),
            ],
            4,
            hysteresis_micros,
            2,
            8,
            77,
        )
        .unwrap()
    }

    fn frame(first: i64, second: i64) -> ContinuousFrame {
        ContinuousFrame::new(vec![
            FixedScalar::from_micros(20, second),
            FixedScalar::from_micros(10, first),
        ])
        .unwrap()
    }

    #[test]
    fn abstraction_is_fixed_point_bounded_and_replay_deterministic() {
        let mut left = DeterministicAbstraction::new(config(0));
        let mut right = DeterministicAbstraction::new(config(0));
        let input = frame(0, 1_000_000);
        let left_result = left.abstract_frame(&input).unwrap();
        let right_result = right.abstract_frame(&input).unwrap();
        assert_eq!(left_result, right_result);
        assert_eq!(left_result.atoms(), &[
            SymbolicAtom { role: 10, bin: 0 },
            SymbolicAtom { role: 20, bin: 3 },
        ]);
        assert_eq!(left_result.state().literals().count(), 8);
        assert_eq!(left_result.hypervector().len(), 2);
        assert_eq!(left_result.hamming_distance(&right_result), Some(0));
    }

    #[test]
    fn hysteresis_suppresses_boundary_flicker_then_allows_real_transition() {
        let mut abstraction = DeterministicAbstraction::new(DeterministicAbstractionConfig::new(
            vec![ChannelRange::new(10, 0, 1_000_000).unwrap()],
            4,
            50_000,
            1,
            4,
            77,
        )
        .unwrap());
        assert_eq!(
            abstraction
                .abstract_frame(&ContinuousFrame::new(vec![FixedScalar::from_micros(
                    10, 249_000
                )])
                .unwrap())
                .unwrap()
                .atoms()[0]
                .bin,
            0
        );
        assert_eq!(
            abstraction
                .abstract_frame(&ContinuousFrame::new(vec![FixedScalar::from_micros(
                    10, 260_000
                )])
                .unwrap())
                .unwrap()
                .atoms()[0]
                .bin,
            0
        );
        assert_eq!(
            abstraction
                .abstract_frame(&ContinuousFrame::new(vec![FixedScalar::from_micros(
                    10, 350_000
                )])
                .unwrap())
                .unwrap()
                .atoms()[0]
                .bin,
            1
        );
    }

    #[test]
    fn malformed_or_partial_frames_fail_closed() {
        assert!(matches!(
            ContinuousFrame::new(vec![
                FixedScalar::from_micros(1, 1),
                FixedScalar::from_micros(1, 2),
            ]),
            Err(AbstractionError::DuplicateRole(1))
        ));
        let mut abstraction = DeterministicAbstraction::new(config(0));
        assert!(matches!(
            abstraction.abstract_frame(
                &ContinuousFrame::new(vec![FixedScalar::from_micros(10, 1)]).unwrap()
            ),
            Err(AbstractionError::MissingRole(20))
        ));
        assert!(matches!(
            abstraction.abstract_frame(
                &ContinuousFrame::new(vec![
                    FixedScalar::from_micros(10, 1),
                    FixedScalar::from_micros(30, 1),
                ])
                .unwrap()
            ),
            Err(AbstractionError::UnknownRole(30))
        ));
        assert!(matches!(
            FixedScalar::from_f64(10, f64::NAN),
            Err(AbstractionError::NonFiniteInput)
        ));
    }

    #[test]
    fn schema_and_state_budgets_are_rejected_before_graph_ingestion() {
        assert!(matches!(
            DeterministicAbstractionConfig::new(
                vec![ChannelRange::new(1, 0, 10).unwrap()],
                4,
                0,
                1,
                3,
                1,
            ),
            Err(AbstractionError::StateBudgetExceeded {
                required: 4,
                limit: 3
            })
        ));
        assert!(matches!(
            DeterministicAbstractionConfig::new(
                vec![ChannelRange::new(1, 0, 10).unwrap()],
                17,
                0,
                1,
                17,
                1,
            ),
            Err(AbstractionError::InvalidBinCount { bins: 17 })
        ));
    }
}
