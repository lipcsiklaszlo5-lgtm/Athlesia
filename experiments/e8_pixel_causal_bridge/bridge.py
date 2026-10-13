"""E8 bounded pixel-to-E6 bridge. No evaluator, hidden-law or world imports.

This is explicit *instrumented* grounding: independent public on/off controls
identify two pixels as Boolean channels; actor motion identifies a gate result.
No automatic predicate discovery, no uninstrumented reset, no ARC-AGI-3 API.
"""
from __future__ import annotations
from dataclasses import dataclass
from agent import Agent as VisualAgent, Demonstration, Frame, find_one, validate
from athlesia.protocol import Edge, Kind, Node, Observation, Relation, Scene, OBSERVE
from athlesia.learner import CannotIdentify as E6CannotIdentify

class GroundingError(ValueError):
    pass

@dataclass(frozen=True)
class Switch:
    pixel:tuple[int,int]
    false_colour:int
    true_colour:int
    off_token:str
    on_token:str

class PixelBridge:
    def __init__(self, demonstrations:tuple[Demonstration,...],
                 controls:tuple[tuple[str,str],...], gate_cell:tuple[int,int]):
        if len(controls)!=2 or len({t for pair in controls for t in pair})!=4:
            raise GroundingError('Need exactly two distinct public on/off control pairs')
        self.visual=VisualAgent()
        self.visual.fit(demonstrations)
        self.controls=controls
        self.gate_cell=gate_cell
        self.switches:tuple[Switch,...]=()
        self.crossing_action=next((token for token,delta in self.visual.action_delta.items()
                                   if delta==(0,1)),None)
        if self.crossing_action is None:
            raise GroundingError('No visually learned right-moving action')
        self.scene=self._make_scene()

    def _make_scene(self)->Scene:
        nodes=[Node(f'obj{i}',Kind.OBJECT) for i in range(3)]
        nodes += [Node('switch0',Kind.FLUENT),Node('switch1',Kind.FLUENT),Node('crossed',Kind.GOAL)]
        nodes += [Node(self.crossing_action,Kind.ACTION)]
        nodes += [Node(token,Kind.ACTION) for pair in self.controls for token in pair]
        edges=[Edge('obj0',Relation.LINK,'obj1'),Edge('obj1',Relation.LINK,'obj2'),
               Edge('switch0',Relation.AT,'obj1'),Edge('switch1',Relation.AT,'obj2'),
               Edge('crossed',Relation.AT,'obj0'),
               Edge(self.crossing_action,Relation.ACTS_ON,'obj0')]
        for i,(off,on) in enumerate(self.controls):
            edges += [Edge(off,Relation.SET_FALSE,f'switch{i}'),
                      Edge(on,Relation.SET_TRUE,f'switch{i}')]
        if len(set(n.id for n in nodes))!=len(nodes):
            raise GroundingError('Action and role token collision')
        layout=tuple((n.id,j%9,j//9) for j,n in enumerate(nodes))
        return Scene(tuple(nodes),tuple(edges),9,9,layout)

    def fit_controls(self,port)->None:
        if self.switches:
            raise GroundingError('Calibration may only run once')
        switches=[]
        for off,on in self.controls:
            before=port.observe_pixels()
            after=port.send_pixels(on)
            if validate(before)!=validate(after):
                raise GroundingError('Calibration grid resized')
            changed=[(r,c) for r,row in enumerate(before) for c,v in enumerate(row)
                     if after[r][c]!=v]
            if len(changed)!=1:
                raise GroundingError('One public control must change exactly one pixel')
            r,c=changed[0]
            false_colour,true_colour=before[r][c],after[r][c]
            restored=port.send_pixels(off)
            if restored!=before or false_colour==true_colour:
                raise GroundingError('Calibration is not reversible')
            if (r,c)==self.gate_cell:
                raise GroundingError('Calibration overlaps declared gate')
            switches.append(Switch((r,c),false_colour,true_colour,off,on))
        if switches[0].pixel==switches[1].pixel:
            raise GroundingError('Two controls refer to the same visual channel')
        self.switches=tuple(switches)

    def pixels_to_values(self,frame:Frame)->tuple[tuple[str,bool],...]:
        validate(frame)
        if len(self.switches)!=2:
            raise GroundingError('Pixel controls have not been calibrated')
        values={}
        for i,s in enumerate(self.switches):
            colour=frame[s.pixel[0]][s.pixel[1]]
            if colour not in (s.false_colour,s.true_colour):
                raise GroundingError('Unidentified Boolean pixel colour')
            values[f'switch{i}']=colour==s.true_colour
        actor=find_one(frame,self.visual.actor_colour)
        values['crossed']=actor==self.gate_cell
        return tuple(sorted(values.items()))

class PublicPixelAdapter:
    """Supply E6 Trace protocol from E7 pixel frames, verifying real actions."""
    def __init__(self,bridge:PixelBridge,world):
        self.bridge=bridge
        self.world=world
        self.actions=0
        self.probe_actions=0
    def observe_pixels(self)->Frame:
        return self.world.observe()
    def send_pixels(self,token:str)->Frame:
        before=self.world.observe()
        after=self.world.send_action(token)
        self.actions+=1
        return after
    def send_action(self,token:str)->Observation:
        if token==OBSERVE:
            frame=self.observe_pixels()
        else:
            prior=self.observe_pixels()
            frame=self.send_pixels(token)
            if token==self.bridge.crossing_action:
                self.probe_actions+=1
                old=find_one(prior,self.bridge.visual.actor_colour)
                actual=find_one(frame,self.bridge.visual.actor_colour)
                delta=self.bridge.visual.action_delta[token]
                expected=(old[0]+delta[0],old[1]+delta[1])
                if actual!=old and actual!=expected:
                    raise GroundingError('Pixel action violates learned motion primitive')
            else:
                switch=next((s for s in self.bridge.switches
                             if token in (s.off_token,s.on_token)),None)
                if switch is None:
                    raise GroundingError('Unknown calibration action')
                changes=[(r,c) for r,row in enumerate(prior) for c,x in enumerate(row)
                         if frame[r][c]!=x]
                if changes not in ([],[switch.pixel]):
                    raise GroundingError('Public control changed unrelated pixels')
        return Observation(self.world.episode,self.world.sequence,self.bridge.scene,
                           self.bridge.pixels_to_values(frame))
