"""E9 public-interface pixel affordance discovery, no simulator imports.

Unlabelled action inventory and real frame transitions reveal two reversible
single-pixel controls and an adjacent blocked target. Encodings are arbitrary.
Not an arbitrary visual object detector or official ARC-AGI-3 agent.
"""
from __future__ import annotations
from dataclasses import dataclass
from collections import defaultdict
from agent import Agent as VisualAgent, Demonstration, Frame, validate, find_one
from athlesia.protocol import Edge, Kind, Node, Observation, Relation, Scene, OBSERVE

class DiscoveryError(ValueError):
    pass

@dataclass(frozen=True)
class DiscoveredControl:
    cell:tuple[int,int]
    false_colour:int
    true_colour:int
    off_token:str
    on_token:str

class DiscoveredBridge:
    def __init__(self, tutorials:tuple[Demonstration,...]):
        self.visual=VisualAgent()
        self.visual.fit(tutorials)
        self.switches:tuple[DiscoveredControl,...]=()
        self.gate:tuple[int,int]|None=None
        self.crossing_action:str|None=None
        self.scene:Scene|None=None
        self.exploration_actions=0

    def discover(self,port)->None:
        if self.scene is not None:
            raise DiscoveryError('Already calibrated')
        before=port.observe()
        h,w=validate(before)
        action_ids=port.available_actions()
        if len(action_ids)!=8 or len(set(action_ids))!=8:
            raise DiscoveryError('Expected eight unique opaque public actions')
        known=set(self.visual.action_delta)
        controls=tuple(sorted(set(action_ids)-known))
        if len(controls)!=4 or not all(a in action_ids for a in known):
            raise DiscoveryError('Unknown public action inventory')
        actor=find_one(before,self.visual.actor_colour)
        classified={self.visual.actor_colour,self.visual.floor_colour,*self.visual.obstacle_colours}
        candidates=[]
        for a,(dr,dc) in self.visual.action_delta.items():
            nr,nc=actor[0]+dr,actor[1]+dc
            if 0<=nr<h and 0<=nc<w and before[nr][nc] not in classified:
                candidates.append(((nr,nc),a))
        if len(candidates)!=1:
            raise DiscoveryError('Ambiguous or absent neighbouring gate candidate')
        self.gate,self.crossing_action=candidates[0]
        # Two full passes are necessary: an idempotent first operation may
        # become informative after its inverse acts later in the same pass.
        observations:dict[str,tuple[tuple[int,int],int]]={}
        frame=before
        for _ in range(2):
            for token in controls:
                after=port.send_action(token)
                self.exploration_actions+=1
                if validate(after)!=(h,w):
                    raise DiscoveryError('Unexpected frame resize')
                changed=[(r,c) for r in range(h) for c in range(w)
                         if frame[r][c]!=after[r][c]]
                if len(changed)>1:
                    raise DiscoveryError('Control has multi-pixel effect')
                if changed:
                    cell=changed[0]
                    if cell in (actor,self.gate):
                        raise DiscoveryError('Control overlaps actor or gate')
                    candidate=(cell,after[cell[0]][cell[1]])
                    if token in observations and observations[token]!=candidate:
                        raise DiscoveryError('Nonstationary pixel-action effect')
                    observations[token]=candidate
                frame=after
        if set(observations)!=set(controls):
            raise DiscoveryError('Cannot identify all controls from two passes')
        grouped=defaultdict(list)
        for action,(cell,value) in observations.items():
            grouped[cell].append((action,value))
        if len(grouped)!=2:
            raise DiscoveryError('Did not discover two distinct switch pixels')
        identified=[]
        for cell,items in sorted(grouped.items()):
            if len(items)!=2 or len({v for _,v in items})!=2:
                raise DiscoveryError('Control pair not reversible and identifiable')
            sorted_vals=sorted((v,t) for t,v in items)
            # False/True are canonical *visual* roles, not the world's polarity.
            (low,off),(high,on)=sorted_vals
            identified.append(DiscoveredControl(cell,low,high,off,on))
        self.switches=tuple(identified)
        self.scene=self._make_scene()

    def _make_scene(self)->Scene:
        assert self.crossing_action is not None
        nodes=[Node(f'obj{i}',Kind.OBJECT) for i in range(3)]
        nodes += [Node('switch0',Kind.FLUENT),Node('switch1',Kind.FLUENT),Node('crossed',Kind.GOAL)]
        nodes += [Node(self.crossing_action,Kind.ACTION)]
        nodes += [Node(t,Kind.ACTION) for s in self.switches for t in (s.off_token,s.on_token)]
        edges=[Edge('obj0',Relation.LINK,'obj1'),Edge('obj1',Relation.LINK,'obj2'),
               Edge('switch0',Relation.AT,'obj1'),Edge('switch1',Relation.AT,'obj2'),
               Edge('crossed',Relation.AT,'obj0'),Edge(self.crossing_action,Relation.ACTS_ON,'obj0')]
        for i,s in enumerate(self.switches):
            edges += [Edge(s.off_token,Relation.SET_FALSE,f'switch{i}'),
                      Edge(s.on_token,Relation.SET_TRUE,f'switch{i}')]
        return Scene(tuple(nodes),tuple(edges),9,9,tuple((n.id,j%9,j//9) for j,n in enumerate(nodes)))

    def pixels_to_values(self,frame:Frame)->tuple[tuple[str,bool],...]:
        validate(frame)
        if self.scene is None or self.gate is None or len(self.switches)!=2:
            raise DiscoveryError('Not calibrated')
        values={}
        for i,s in enumerate(self.switches):
            colour=frame[s.cell[0]][s.cell[1]]
            if colour not in (s.false_colour,s.true_colour):
                raise DiscoveryError('Unknown visual state value')
            values[f'switch{i}']=colour==s.true_colour
        values['crossed']=find_one(frame,self.visual.actor_colour)==self.gate
        return tuple(sorted(values.items()))

class PublicAdapter:
    def __init__(self,bridge:DiscoveredBridge,world):
        self.bridge,self.world=bridge,world
        self.actions=0
        self.probes=0

    def send_action(self,token:str)->Observation:
        if self.bridge.scene is None:
            raise DiscoveryError('Uncalibrated bridge')
        if token==OBSERVE:
            frame=self.world.observe()
        else:
            before=self.world.observe()
            frame=self.world.send_action(token)
            self.actions+=1
            changed=[(r,c) for r,row in enumerate(before) for c,v in enumerate(row)
                     if v!=frame[r][c]]
            if token==self.bridge.crossing_action:
                self.probes+=1
                a=find_one(before,self.bridge.visual.actor_colour)
                b=find_one(frame,self.bridge.visual.actor_colour)
                dr,dc=self.bridge.visual.action_delta[token]
                if b not in (a,(a[0]+dr,a[1]+dc)):
                    raise DiscoveryError('Unexpected actor displacement')
            else:
                chosen=next((s for s in self.bridge.switches if token in (s.off_token,s.on_token)),None)
                if chosen is None or changed not in ([],[chosen.cell]):
                    raise DiscoveryError('Unexpected unlearned control effect')
        return Observation(self.world.episode,self.world.sequence,
                           self.bridge.scene,self.bridge.pixels_to_values(frame))
