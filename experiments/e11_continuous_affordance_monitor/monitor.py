"""E11 runtime monitor for E9 learned pixel affordances.

Finite evidence cannot certify future stationarity. This module makes no claim
of universal stability; it checks every executed control transition, records
counterexamples, quarantines a contradicted model, and refuses further actions.
No private simulator imports. The existing E9 discoverer remains unchanged.
"""
from __future__ import annotations
from dataclasses import dataclass
from agent import Frame, validate

class ModelQuarantined(RuntimeError):
    pass

@dataclass(frozen=True)
class Counterexample:
    token: str
    action_number: int
    reason: str
    before: Frame
    after: Frame | None

class ContinuousAffordanceMonitor:
    """Monitor E9 discovered switch transitions, never certify infinite stability."""
    def __init__(self, bridge, port):
        if bridge.scene is None or len(bridge.switches) != 2:
            raise ValueError('Require identified E9 bridge')
        self.bridge = bridge
        self.port = port
        self.actions = 0
        self.confirmed_actions = 0
        self.status = 'provisional'
        self.counterexample: Counterexample | None = None
        self.history: list[tuple[str,Frame,Frame]] = []
        self.control_by_token = {
            tok: (control, colour)
            for control in bridge.switches
            for tok, colour in ((control.off_token,control.false_colour),
                                (control.on_token,control.true_colour))
        }
        if len(self.control_by_token) != 4:
            raise ValueError('Ambiguous action vocabulary')

    def _quarantine(self, token: str, reason: str, before: Frame,
                    after: Frame | None = None) -> None:
        self.status = 'quarantined'
        self.counterexample = Counterexample(token,self.actions,reason,before,after)
        raise ModelQuarantined(reason)

    def execute(self, token: str) -> Frame:
        if self.status == 'quarantined':
            raise ModelQuarantined('Model already quarantined; no further interactions')
        if token not in self.control_by_token:
            raise ValueError('Only previously identified control actions are monitored')
        before = self.port.observe()
        control, expected_colour = self.control_by_token[token]
        try:
            h,w = validate(before)
            if any(before[c.cell[0]][c.cell[1]] not in
                   (c.false_colour,c.true_colour) for c in self.bridge.switches):
                self._quarantine(token,'unidentified_current_visual_state',before)
        except (IndexError, TypeError, ValueError) as ex:
            self._quarantine(token,'invalid_pre_action_observation:'+type(ex).__name__,before)
        # Charge an attempted action even if the port fails after dispatch.
        self.actions += 1
        try:
            after = self.port.send_action(token)
        except Exception as ex:
            self._quarantine(token,'action_port_failed:'+type(ex).__name__,before)
        try:
            if validate(after) != (h,w):
                self._quarantine(token,'frame_dimensions_changed',before,after)
            if after[control.cell[0]][control.cell[1]] != expected_colour:
                self._quarantine(token,'predicted_control_effect_falsified',before,after)
            changed = [(r,c) for r in range(h) for c in range(w)
                       if before[r][c] != after[r][c]]
            if changed not in ([], [control.cell]):
                self._quarantine(token,'unpredicted_visual_side_effect',before,after)
            if any(after[c.cell[0]][c.cell[1]] not in
                   (c.false_colour,c.true_colour) for c in self.bridge.switches):
                self._quarantine(token,'unidentified_successor_state',before,after)
        except (IndexError, TypeError, ValueError) as ex:
            self._quarantine(token,'invalid_successor:'+type(ex).__name__,before,after)
        self.confirmed_actions += 1
        self.history.append((token,before,after))
        # A finite number of consistent actions does NOT make a timeless law.
        return after

    def exercise(self, rounds: int) -> None:
        if not 0 <= rounds <= 50:
            raise ValueError('Bounded monitoring rounds required')
        for _ in range(rounds):
            for control in self.bridge.switches:
                self.execute(control.on_token)
                self.execute(control.off_token)
