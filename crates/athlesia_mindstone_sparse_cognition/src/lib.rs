pub const COGNITIVE_SIGNAL_SCALE: u16 = 1000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CognitiveSignal(u16);

impl CognitiveSignal {
    pub fn new(value: u16) -> Option<Self> {
        if value > COGNITIVE_SIGNAL_SCALE {
            return None;
        }

        Some(Self(value))
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn maximum() -> Self {
        Self(COGNITIVE_SIGNAL_SCALE)
    }

    pub fn value(self) -> u16 {
        self.0
    }

    pub fn learning_progress(previous_uncertainty: Self, current_uncertainty: Self) -> Self {
        if previous_uncertainty > current_uncertainty {
            Self(previous_uncertainty.0.saturating_sub(current_uncertainty.0))
        } else {
            Self::zero()
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneSignalProfile {
    surprise: CognitiveSignal,
    uncertainty: CognitiveSignal,
    novelty: CognitiveSignal,
    learning_progress: CognitiveSignal,
    information_gain: CognitiveSignal,
}

impl MindstoneSignalProfile {
    pub fn new(
        surprise: CognitiveSignal,
        uncertainty: CognitiveSignal,
        novelty: CognitiveSignal,
        learning_progress: CognitiveSignal,
        information_gain: CognitiveSignal,
    ) -> Self {
        Self {
            surprise,
            uncertainty,
            novelty,
            learning_progress,
            information_gain,
        }
    }

    pub fn surprise(self) -> CognitiveSignal {
        self.surprise
    }

    pub fn uncertainty(self) -> CognitiveSignal {
        self.uncertainty
    }

    pub fn novelty(self) -> CognitiveSignal {
        self.novelty
    }

    pub fn learning_progress_signal(self) -> CognitiveSignal {
        self.learning_progress
    }

    pub fn information_gain(self) -> CognitiveSignal {
        self.information_gain
    }

    pub fn salience(self) -> CognitiveSalience {
        let values = [
            self.surprise,
            self.uncertainty,
            self.novelty,
            self.learning_progress,
            self.information_gain,
        ];

        let total = values
            .iter()
            .map(|signal| u32::from(signal.value()))
            .sum::<u32>();

        let peak = values
            .iter()
            .map(|signal| u32::from(signal.value()))
            .max()
            .unwrap_or(0);

        let composite = (peak.saturating_mul(2).saturating_add(total)) / 7;

        CognitiveSalience(composite as u16)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CognitiveSalience(u16);

impl CognitiveSalience {
    pub fn value(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CognitiveBudget {
    units: u32,
}

impl CognitiveBudget {
    pub fn new(units: u32) -> Option<Self> {
        if units == 0 {
            return None;
        }

        Some(Self { units })
    }

    pub fn units(self) -> u32 {
        self.units
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CognitiveAdmissionClass {
    Ignore,
    CheapUpdate,
    Deliberate,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SparseCognitionPolicy {
    cheap_threshold: CognitiveSignal,
    deliberate_threshold: CognitiveSignal,
    cheap_compute_units: u32,
    deliberate_compute_units: u32,
}

impl SparseCognitionPolicy {
    pub fn new(
        cheap_threshold: CognitiveSignal,
        deliberate_threshold: CognitiveSignal,
        cheap_compute_units: u32,
        deliberate_compute_units: u32,
    ) -> Option<Self> {
        if cheap_threshold >= deliberate_threshold {
            return None;
        }

        if cheap_compute_units == 0 {
            return None;
        }

        if deliberate_compute_units < cheap_compute_units {
            return None;
        }

        Some(Self {
            cheap_threshold,
            deliberate_threshold,
            cheap_compute_units,
            deliberate_compute_units,
        })
    }

    pub fn cheap_threshold(self) -> CognitiveSignal {
        self.cheap_threshold
    }

    pub fn deliberate_threshold(self) -> CognitiveSignal {
        self.deliberate_threshold
    }

    pub fn cheap_compute_units(self) -> u32 {
        self.cheap_compute_units
    }

    pub fn deliberate_compute_units(self) -> u32 {
        self.deliberate_compute_units
    }

    pub fn classify(self, profile: MindstoneSignalProfile) -> CognitiveAdmissionClass {
        let salience = profile.salience().value();

        if salience < self.cheap_threshold.value() {
            CognitiveAdmissionClass::Ignore
        } else if salience < self.deliberate_threshold.value() {
            CognitiveAdmissionClass::CheapUpdate
        } else {
            CognitiveAdmissionClass::Deliberate
        }
    }

    pub fn admit(
        self,
        profile: MindstoneSignalProfile,
        budget: CognitiveBudget,
    ) -> CognitiveAdmissionDecision {
        let salience = profile.salience();

        let class = self.classify(profile);

        let requested_units = match class {
            CognitiveAdmissionClass::Ignore => 0,
            CognitiveAdmissionClass::CheapUpdate => self.cheap_compute_units,
            CognitiveAdmissionClass::Deliberate => self.deliberate_compute_units,
        };

        let granted_units = requested_units.min(budget.units());

        CognitiveAdmissionDecision {
            class,
            salience,
            requested_units,
            granted_units,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CognitiveAdmissionDecision {
    class: CognitiveAdmissionClass,
    salience: CognitiveSalience,
    requested_units: u32,
    granted_units: u32,
}

impl CognitiveAdmissionDecision {
    pub fn class(self) -> CognitiveAdmissionClass {
        self.class
    }

    pub fn salience(self) -> CognitiveSalience {
        self.salience
    }

    pub fn requested_units(self) -> u32 {
        self.requested_units
    }

    pub fn granted_units(self) -> u32 {
        self.granted_units
    }

    pub fn is_admitted(self) -> bool {
        self.class != CognitiveAdmissionClass::Ignore
    }

    pub fn is_deliberative(self) -> bool {
        self.class == CognitiveAdmissionClass::Deliberate
    }

    pub fn is_budget_limited(self) -> bool {
        self.granted_units < self.requested_units
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MindstoneSparseCognition;

impl MindstoneSparseCognition {
    pub fn evaluate(
        profile: MindstoneSignalProfile,
        policy: SparseCognitionPolicy,
        budget: CognitiveBudget,
    ) -> CognitiveAdmissionDecision {
        policy.admit(profile, budget)
    }
}
