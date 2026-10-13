"""Private E8 test world; never imported by the E8 bridge/solver.

Two instrumented switch indicators are rendered as pixels. An opaque movement
action crosses a one-step gate iff a hidden Boolean conjunction is satisfied.
The switch controls are publicly advertised; the hidden conjunction is not.
"""
from __future__ import annotations
from dataclasses import dataclass
from agent import Frame

@dataclass(frozen=True)
class GateSpec:
    case: object
    law: tuple[tuple[int, bool], ...]
    on_colour: int
    off_colour: int
    control_tokens: tuple[tuple[str, str], ...]
    crossing_action: str

    @classmethod
    def create(cls, case, law):
        palette=set(case.palette)
        extras=[v for v in range(1,256) if v not in palette]
        tokens=tuple((f'ctl_{x}_{y}_{case.action_map[0][0][7:]}', f'ctl_{x}_{z}_{case.action_map[0][0][7:]}') for x,(y,z) in enumerate((('off','on'),('off','on'))))
        # pair order is (off, on); opaque token strings are controls, not rule labels
        right=next(token for token, delta in case.action_map if delta==(0,1))
        return cls(case,tuple(law),extras[0],extras[1],tokens,right)

class GateWorld:
    SWITCH_CELLS=((0,1),(0,2))
    START=(4,3)
    GATE=(4,4)
    def __init__(self,spec:GateSpec, episode:int, flags=(False,False)):
        self.spec=spec
        self.episode=f'ep_e8_{episode}'
        self.switches=list(flags)
        self.actor=self.START
        self.sequence=0
        self.real_actions=0
        self._movement=dict(spec.case.action_map)

    def observe(self)->Frame:
        size=9
        floor,wall,actor,marker=self.spec.case.palette
        rows=[[wall if r in (0,size-1) or c in (0,size-1) else floor for c in range(size)] for r in range(size)]
        for i,(r,c) in enumerate(self.SWITCH_CELLS):
            rows[r][c]=self.spec.on_colour if self.switches[i] else self.spec.off_colour
        rows[self.GATE[0]][self.GATE[1]]=marker
        rows[self.actor[0]][self.actor[1]]=actor
        return tuple(tuple(row) for row in rows)

    def send_action(self,token:str)->Frame:
        changed=False
        for i,(off,on) in enumerate(self.spec.control_tokens):
            if token==off:
                self.switches[i]=False
                changed=True
                break
            if token==on:
                self.switches[i]=True
                changed=True
                break
        if not changed:
            if token not in self._movement:
                raise ValueError('Invalid public action')
            dr,dc=self._movement[token]
            nxt=(self.actor[0]+dr,self.actor[1]+dc)
            # The benchmark probes only the crossing move. A different movement
            # remains legal but irrelevant; it can never be mistaken for a gate.
            open_gate=all(self.switches[i]==v for i,v in self.spec.law)
            if 1<=nxt[0]<8 and 1<=nxt[1]<8:
                if nxt!=self.GATE or open_gate:
                    self.actor=nxt
        self.real_actions+=1
        self.sequence+=1
        return self.observe()
