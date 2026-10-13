"""PRIVATE E9 same-author image world. Never imported by the bridge/learner."""
from __future__ import annotations
from dataclasses import dataclass
import random
from agent import Frame

Coord = tuple[int,int]

@dataclass(frozen=True)
class HiddenSpec:
    case: object
    law: tuple[tuple[int,bool], ...]
    pixels: tuple[Coord,Coord]
    colours: tuple[tuple[int,int],tuple[int,int]] # physical False,True
    control_pairs: tuple[tuple[str,str],tuple[str,str]]
    action_inventory: tuple[str,...]
    start: Coord
    gate: Coord
    decorations: tuple[tuple[Coord,int],...]

    @classmethod
    def create(cls,case,law,seed:int):
        rng=random.Random(seed^0xE9102026)
        used=set(case.palette)
        extras=rng.sample([v for v in range(1,256) if v not in used],6)
        colours=((extras[0],extras[1]),(extras[2],extras[3]))
        positions=rng.sample([(0,c) for c in range(1,8)],2)
        rr,cc=rng.randrange(3,6),rng.randrange(2,5)
        start=(rr,cc);gate=(rr,cc+1)
        pair=tuple(tuple(f'ux_{rng.getrandbits(62):016x}' for _ in range(2)) for __ in range(2))
        inventory=[t for x in pair for t in x]+[t for t,_ in case.action_map]
        rng.shuffle(inventory)
        decorations=(((1,1),extras[4]),((7,7),extras[5]))
        return cls(case,tuple(law),tuple(positions),colours,pair,tuple(inventory),start,gate,decorations)

class World:
    def __init__(self,spec:HiddenSpec,episode:int,flags=(False,False)):
        self._spec=spec
        self._flags=list(flags)
        self._pos=spec.start
        self.episode=f'ep_e9_{episode}'
        self.sequence=0
        self.real_actions=0
        self._movement=dict(spec.case.action_map)

    def available_actions(self)->tuple[str,...]:
        return self._spec.action_inventory

    def observe(self)->Frame:
        s=9
        floor,wall,actor,marker=self._spec.case.palette
        rows=[[wall if r in (0,s-1) or c in (0,s-1) else floor for c in range(s)] for r in range(s)]
        for p,v in self._spec.decorations:
            rows[p[0]][p[1]]=v
        for i,(r,c) in enumerate(self._spec.pixels):
            rows[r][c]=self._spec.colours[i][int(self._flags[i])]
        r,c=self._spec.gate
        rows[r][c]=marker
        r,c=self._pos
        rows[r][c]=actor
        return tuple(tuple(row) for row in rows)

    def send_action(self,token:str)->Frame:
        found=False
        for i,(off,on) in enumerate(self._spec.control_pairs):
            if token in (off,on):
                self._flags[i]=(token==on)
                found=True
                break
        if not found:
            if token not in self._movement:
                raise ValueError('Unknown action')
            dr,dc=self._movement[token]
            nr,nc=self._pos[0]+dr,self._pos[1]+dc
            if 1<=nr<8 and 1<=nc<8:
                blocked=(nr,nc)==self._spec.gate and not all(self._flags[i]==v for i,v in self._spec.law)
                if not blocked:
                    self._pos=(nr,nc)
        self.real_actions+=1
        self.sequence+=1
        return self.observe()
