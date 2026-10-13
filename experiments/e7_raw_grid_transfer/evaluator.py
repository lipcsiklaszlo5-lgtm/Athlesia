"""E7 private, same-author grid environment. Solver cannot import this module.

The public interface exposes only raw integer pixels and opaque action tokens.
No object graph, semantic cell type, compass labels or Boolean channel roles.
The target coordinate is externally declared to the agent; no goal discovery.
"""
from __future__ import annotations

import random
from collections import deque
from dataclasses import dataclass, replace

from agent import Demonstration, Frame

Direction = tuple[int, int]
DIRECTIONS: tuple[Direction, ...] = ((1,0),(-1,0),(0,1),(0,-1))


@dataclass(frozen=True)
class HiddenCase:
    palette: tuple[int, int, int, int]  # floor, wall, actor, target marker
    action_map: tuple[tuple[str, Direction], ...]
    walls: frozenset[tuple[int,int]]
    start: tuple[int,int]
    goal: tuple[int,int]
    size: int


class World:
    def __init__(self, case: HiddenCase, pos: tuple[int,int] | None = None,
                 walls: frozenset[tuple[int,int]] | None = None):
        self._case=case
        self._pos=case.start if pos is None else pos
        self._walls=case.walls if walls is None else walls
        self.calls=0
        self._actions=dict(case.action_map)

    def observe(self) -> Frame:
        floor, wall, actor, marker=self._case.palette
        s=self._case.size
        grid=[[wall if (r,c) in self._walls else floor
               for c in range(s)] for r in range(s)]
        gr,gc=self._case.goal
        grid[gr][gc]=marker
        pr,pc=self._pos
        grid[pr][pc]=actor
        return tuple(tuple(row) for row in grid)

    def send_action(self, token: str) -> Frame:
        dr,dc=self._actions[token]
        nxt=(self._pos[0]+dr,self._pos[1]+dc)
        if nxt not in self._walls and 0<=nxt[0]<self._case.size and 0<=nxt[1]<self._case.size:
            self._pos=nxt
        self.calls+=1
        return self.observe()

    @property
    def success(self) -> bool:
        return self._pos == self._case.goal


def _distance(size, walls, start, goal):
    queue=deque([(start,0)])
    seen={start}
    while queue:
        p,d=queue.popleft()
        if p==goal:
            return d
        for dr,dc in DIRECTIONS:
            nxt=(p[0]+dr,p[1]+dc)
            if (0<=nxt[0]<size and 0<=nxt[1]<size
                    and nxt not in walls and nxt not in seen):
                seen.add(nxt)
                queue.append((nxt,d+1))
    return None


def new_case(seed: int, size: int=9) -> tuple[HiddenCase,int]:
    rng=random.Random(seed)
    palette=tuple(rng.sample(range(1,256),4))
    tokens=[f'opaque_{rng.getrandbits(56):014x}' for _ in range(4)]
    vectors=list(DIRECTIONS)
    rng.shuffle(vectors)
    action_map=tuple(zip(tokens,vectors))
    border=frozenset((r,c) for r in range(size) for c in range(size)
                      if r in (0,size-1) or c in (0,size-1))
    interior=[(r,c) for r in range(1,size-1) for c in range(1,size-1)]
    for _ in range(200):
        start,goal=rng.sample(interior,2)
        walls=frozenset(set(border)|{p for p in interior if p not in (start,goal) and rng.random()<0.22})
        dist=_distance(size,walls,start,goal)
        if dist is not None and 5<=dist<=26:
            return HiddenCase(palette,action_map,walls,start,goal,size),dist
    raise AssertionError('Could not generate reachable held-out map')


def training_demonstrations(case: HiddenCase) -> tuple[Demonstration,...]:
    # A separate clear training room using the SAME palette/opaque tokens as the
    # held-out maze. The evaluator does not reveal actor or obstacle roles.
    size=case.size
    border=frozenset((r,c) for r in range(size) for c in range(size)
                      if r in (0,size-1) or c in (0,size-1))
    tutorial_case=replace(case, goal=(1, size-2))
    traces=[]
    for token,_ in case.action_map:
        w=World(tutorial_case,(size//2,size//2),border)
        before=w.observe()
        after=w.send_action(token)
        traces.append(Demonstration(before,token,after))
    # One real collision, with a known action token, grounds a blocked colour.
    upward=next(token for token,delta in case.action_map if delta==(-1,0))
    blocked=World(tutorial_case,(1,1),border)
    before=blocked.observe()
    after=blocked.send_action(upward)
    traces.append(Demonstration(before,upward,after))
    return tuple(traces)
