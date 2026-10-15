//! R3E-C research primitive: bounded, evidence-derived structural role shifts.
//!
//! No ARC action IDs, cell coordinates, colour increments, or test outcomes are
//! encoded here. Numeric shifts and changing structural positions are inferred
//! from contrasting retained transition episodes. This is a proposed transfer
//! hypothesis, not permission to execute and not yet wired to M51 authority.
use super::{CognitiveStructure, GroundedStateSnapshot, GroundedTransformationEpisode};

const MAX_EPISODES: usize = 256;
const MAX_FACTS: usize = 64;
const MAX_PATTERNS: usize = 256;
const MAX_DEPTH: usize = 12;
const MIN_DISTINCT_SOURCES: usize = 3;
const MIN_SUPPORT: usize = 3;
const MIN_PRECISION_PER_MILLE: usize = 900;
const MIN_LIFT_PER_MILLE: usize = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Role {
    Fixed(u64),
    Shift(i128),
    Ordered(Vec<Role>),
    Unordered(Vec<Role>),
}

impl Role {
    fn derive(
        before: &CognitiveStructure,
        after: &CognitiveStructure,
        depth: usize,
    ) -> Option<(Self, usize, usize)> {
        if depth > MAX_DEPTH {
            return None;
        }
        match (before, after) {
            (CognitiveStructure::Atom(a), CognitiveStructure::Atom(b)) if a == b => {
                Some((Self::Fixed(*a), 0, 1))
            }
            (CognitiveStructure::Atom(a), CognitiveStructure::Atom(b)) => {
                Some((Self::Shift(i128::from(*b) - i128::from(*a)), 1, 0))
            }
            (CognitiveStructure::Ordered(a), CognitiveStructure::Ordered(b))
            | (CognitiveStructure::Unordered(a), CognitiveStructure::Unordered(b)) => {
                if a.len() != b.len() || a.is_empty() {
                    return None;
                }
                let mut children = Vec::with_capacity(a.len());
                let (mut changing, mut stable) = (0, 0);
                for (x, y) in a.iter().zip(b) {
                    let (role, changes, invariants) = Self::derive(x, y, depth + 1)?;
                    changing += changes;
                    stable += invariants;
                    children.push(role);
                }
                let role = if matches!(before, CognitiveStructure::Ordered(_)) {
                    Self::Ordered(children)
                } else {
                    Self::Unordered(children)
                };
                Some((role, changing, stable))
            }
            _ => None,
        }
    }

    fn matches(&self, input: &CognitiveStructure) -> bool {
        match (self, input) {
            (Self::Fixed(a), CognitiveStructure::Atom(b)) => a == b,
            (Self::Shift(_), CognitiveStructure::Atom(_)) => true,
            (Self::Ordered(a), CognitiveStructure::Ordered(b))
            | (Self::Unordered(a), CognitiveStructure::Unordered(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.matches(y))
            }
            _ => false,
        }
    }

    fn apply(&self, input: &CognitiveStructure) -> Option<CognitiveStructure> {
        match (self, input) {
            (Self::Fixed(a), CognitiveStructure::Atom(b)) if a == b => {
                Some(CognitiveStructure::atom(*a))
            }
            (Self::Shift(delta), CognitiveStructure::Atom(value)) => {
                let next = i128::from(*value).checked_add(*delta)?;
                Some(CognitiveStructure::atom(u64::try_from(next).ok()?))
            }
            (Self::Ordered(a), CognitiveStructure::Ordered(b))
            | (Self::Unordered(a), CognitiveStructure::Unordered(b))
                if a.len() == b.len() =>
            {
                let children = a
                    .iter()
                    .zip(b)
                    .map(|(x, y)| x.apply(y))
                    .collect::<Option<Vec<_>>>()?;
                if matches!(self, Self::Ordered(_)) {
                    CognitiveStructure::ordered(children)
                } else {
                    CognitiveStructure::unordered(children)
                }
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Candidate {
    role: Role,
    source_samples: Vec<CognitiveStructure>,
}

impl Candidate {
    fn new(role: Role) -> Self {
        Self {
            role,
            source_samples: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalEffectTransferPrediction {
    additions: Vec<CognitiveStructure>,
    removals: Vec<CognitiveStructure>,
    applicable_patterns: usize,
    minimum_precision_per_mille: u16,
    minimum_association_lift_per_mille: u16,
}

impl RelationalEffectTransferPrediction {
    pub fn additions(&self) -> &[CognitiveStructure] {
        &self.additions
    }
    pub fn removals(&self) -> &[CognitiveStructure] {
        &self.removals
    }
    pub fn applicable_patterns(&self) -> usize {
        self.applicable_patterns
    }
    pub fn minimum_precision_per_mille(&self) -> u16 {
        self.minimum_precision_per_mille
    }
    pub fn minimum_association_lift_per_mille(&self) -> u16 {
        self.minimum_association_lift_per_mille
    }
}

/// Derive a role-rebound effect solely from existing experience, and evaluate
/// it on an unobserved source state. Returns None when evidence is ambiguous,
/// confounded, truncated, inconsistent, or cannot be grounded structurally.
/// Zero-shot probe observations are never supplied to this function.
pub fn predict_relational_effect_transfer(
    episodes: &[GroundedTransformationEpisode],
    source: &GroundedStateSnapshot,
    action: &CognitiveStructure,
) -> Option<RelationalEffectTransferPrediction> {
    if episodes.is_empty()
        || episodes.len() > MAX_EPISODES
        || source.fact_count() > MAX_FACTS
        || episodes
            .iter()
            .any(|e| e.before().fact_count() > MAX_FACTS || e.after().fact_count() > MAX_FACTS)
    {
        return None;
    }

    let mut candidates: Vec<Candidate> = Vec::new();
    for episode in episodes.iter().filter(|e| e.transformation() == action) {
        let removed: Vec<_> = episode
            .before()
            .facts()
            .iter()
            .filter(|f| !episode.after().contains_fact(f))
            .collect();
        let added: Vec<_> = episode
            .after()
            .facts()
            .iter()
            .filter(|f| !episode.before().contains_fact(f))
            .collect();
        for old in removed {
            // One *unique* least-changing structural correspondence is needed.
            // This refuses ambiguous pairings instead of attributing a nearby
            // object's outcome to the wrong physical/structural source.
            let mut best: Option<(usize, Role)> = None;
            let mut ambiguous = false;
            for new in &added {
                let Some((role, changed, stable)) = Role::derive(old, new, 0) else {
                    continue;
                };
                if changed == 0 || stable == 0 {
                    continue;
                }
                match &best {
                    None => {
                        best = Some((changed, role));
                        ambiguous = false;
                    }
                    Some((score, _)) if changed < *score => {
                        best = Some((changed, role));
                        ambiguous = false;
                    }
                    Some((score, _)) if changed == *score => {
                        ambiguous = true;
                    }
                    _ => {}
                }
            }
            let Some((_, role)) = best else {
                continue;
            };
            if ambiguous {
                continue;
            }
            let i = match candidates
                .iter()
                .position(|candidate| candidate.role == role)
            {
                Some(index) => index,
                None => {
                    if candidates.len() >= MAX_PATTERNS {
                        return None;
                    }
                    candidates.push(Candidate::new(role));
                    candidates.len() - 1
                }
            };
            if !candidates[i].source_samples.contains(old) {
                candidates[i].source_samples.push((*old).clone());
            }
        }
    }

    let mut admissible = Vec::<(Role, usize, usize)>::new();
    for candidate in candidates {
        if candidate.source_samples.len() < MIN_DISTINCT_SOURCES {
            continue;
        }
        let (mut action_support, mut action_opportunity) = (0_usize, 0_usize);
        let (mut global_support, mut global_opportunity) = (0_usize, 0_usize);
        for episode in episodes {
            for before in episode.before().facts() {
                if !candidate.role.matches(before) {
                    continue;
                }
                let Some(expected) = candidate.role.apply(before) else {
                    return None;
                };
                let observed = !episode.after().contains_fact(before)
                    && episode.after().contains_fact(&expected);
                global_opportunity += 1;
                if observed {
                    global_support += 1;
                }
                if episode.transformation() == action {
                    action_opportunity += 1;
                    if observed {
                        action_support += 1;
                    }
                }
            }
        }
        if action_support < MIN_SUPPORT || action_opportunity == 0 || global_opportunity == 0 {
            continue;
        }
        let precision = action_support * 1000 / action_opportunity;
        let baseline = global_support * 1000 / global_opportunity;
        let lift = precision.saturating_sub(baseline);
        if precision < MIN_PRECISION_PER_MILLE || lift < MIN_LIFT_PER_MILLE {
            continue;
        }
        admissible.push((candidate.role, precision, lift));
    }

    let mut predictions = Vec::<(CognitiveStructure, CognitiveStructure)>::new();
    let mut used = Vec::<usize>::new();
    for before in source.facts() {
        for (i, (role, _, _)) in admissible.iter().enumerate() {
            if !role.matches(before) {
                continue;
            }
            let after = role.apply(before)?;
            if &after == before {
                continue;
            }
            if let Some((_, old_result)) = predictions.iter().find(|(a, _)| a == before) {
                // Multiple learned hypotheses with conflicting consequences
                // cannot silently compete at the held-out decision boundary.
                if old_result != &after {
                    return None;
                }
            } else {
                predictions.push((before.clone(), after));
            }
            if !used.contains(&i) {
                used.push(i);
            }
        }
    }
    if predictions.is_empty() {
        return None;
    }
    let mut additions = predictions
        .iter()
        .map(|(_, b)| b.clone())
        .collect::<Vec<_>>();
    let mut removals = predictions
        .iter()
        .map(|(a, _)| a.clone())
        .collect::<Vec<_>>();
    additions.sort();
    additions.dedup();
    removals.sort();
    removals.dedup();
    if additions.iter().any(|f| removals.contains(f)) {
        return None;
    }
    let min_precision = used.iter().map(|i| admissible[*i].1).min()?;
    let min_lift = used.iter().map(|i| admissible[*i].2).min()?;
    Some(RelationalEffectTransferPrediction {
        additions,
        removals,
        applicable_patterns: used.len(),
        minimum_precision_per_mille: min_precision as u16,
        minimum_association_lift_per_mille: min_lift as u16,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn a(v: u64) -> CognitiveStructure {
        CognitiveStructure::atom(v)
    }
    fn ordered(items: Vec<CognitiveStructure>) -> CognitiveStructure {
        CognitiveStructure::ordered(items).unwrap()
    }
    fn unordered(items: Vec<CognitiveStructure>) -> CognitiveStructure {
        CognitiveStructure::unordered(items).unwrap()
    }
    fn cell(x: u64, value: u64) -> CognitiveStructure {
        ordered(vec![a(0xC311), a(x), a(0), a(value)])
    }
    fn state(value: u64) -> GroundedStateSnapshot {
        let c0 = cell(0, value);
        let c1 = cell(1, value);
        let group = unordered(vec![c0.clone(), c1.clone()]);
        GroundedStateSnapshot::new(vec![c0, c1, group.clone(), unordered(vec![group])]).unwrap()
    }
    fn episode(before: u64, after: u64, action: u64) -> GroundedTransformationEpisode {
        GroundedTransformationEpisode::new(state(before), state(after), a(action))
    }
    fn evidence() -> Vec<GroundedTransformationEpisode> {
        let mut episodes = Vec::new();
        for before in [1, 2, 3] {
            for _ in 0..3 {
                episodes.push(episode(before, before + 1, 1));
                episodes.push(episode(before, before, 2));
            }
        }
        episodes
    }
    fn exact(pred: &RelationalEffectTransferPrediction, old: u64, new: u64) {
        let before = state(old);
        let after = state(new);
        let additions = after
            .facts()
            .iter()
            .filter(|f| !before.contains_fact(f))
            .cloned()
            .collect::<Vec<_>>();
        let removals = before
            .facts()
            .iter()
            .filter(|f| !after.contains_fact(f))
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(pred.additions(), additions.as_slice());
        assert_eq!(pred.removals(), removals.as_slice());
    }
    #[test]
    fn r3ec_unseen_sources_have_exact_transferred_effects() {
        let learned = evidence();
        for (old, new) in [(5, 6), (6, 7)] {
            let pred = predict_relational_effect_transfer(&learned, &state(old), &a(1))
                .expect("three independent sources and contrast permit a grounded hypothesis");
            exact(&pred, old, new);
            assert_eq!(pred.additions().len(), 4);
            assert_eq!(pred.removals().len(), 4);
            assert!(pred.applicable_patterns() >= 4);
            assert!(pred.minimum_association_lift_per_mille() > 0);
        }
        assert_eq!(learned.len(), 18, "holdout never enters retained evidence");
    }
    #[test]
    fn r3ec_constant_target_is_not_misrepresented_as_relative_change() {
        let mut episodes = Vec::new();
        for before in [1, 3, 4] {
            for _ in 0..3 {
                episodes.push(episode(before, 2, 1));
                episodes.push(episode(before, before, 2));
            }
        }
        assert!(predict_relational_effect_transfer(&episodes, &state(5), &a(1)).is_none());
    }
    #[test]
    fn r3ec_no_interventional_contrast_does_not_claim_causality() {
        let episodes: Vec<_> = [1, 2, 3]
            .into_iter()
            .flat_map(|before| {
                [
                    episode(before, before + 1, 1),
                    episode(before, before + 1, 2),
                    episode(before, before + 1, 1),
                    episode(before, before + 1, 2),
                ]
            })
            .collect();
        assert!(predict_relational_effect_transfer(&episodes, &state(5), &a(1)).is_none());
    }
    #[test]
    fn r3ec_insufficient_distinct_sources_cannot_extrapolate() {
        let episodes: Vec<_> = [1, 2]
            .into_iter()
            .flat_map(|before| {
                [
                    episode(before, before + 1, 1),
                    episode(before, before, 2),
                    episode(before, before + 1, 1),
                    episode(before, before, 2),
                ]
            })
            .collect();
        assert!(predict_relational_effect_transfer(&episodes, &state(5), &a(1)).is_none());
    }
    #[test]
    fn r3ec_wrong_action_or_conflicting_effect_is_rejected() {
        let learned = evidence();
        assert!(predict_relational_effect_transfer(&learned, &state(5), &a(2)).is_none());
        let mut conflict = learned.clone();
        for before in [1, 2, 3] {
            for _ in 0..3 {
                conflict.push(episode(before, before + 2, 1));
            }
        }
        assert!(predict_relational_effect_transfer(&conflict, &state(5), &a(1)).is_none());
    }
    #[test]
    fn r3ec_evidence_order_is_irrelevant() {
        let first = evidence();
        let mut reverse = first.clone();
        reverse.reverse();
        assert_eq!(
            predict_relational_effect_transfer(&first, &state(5), &a(1)),
            predict_relational_effect_transfer(&reverse, &state(5), &a(1))
        );
    }
}
