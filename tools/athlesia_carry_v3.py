#!/usr/bin/env python3
"""Observed-interaction extension of the small ARC example learner."""
import collections

import athlesia_example_agent_v2 as base


def area(obj):
    return (obj[2] - obj[0] + 1) * (obj[3] - obj[1] + 1)


class CarryLearner(base.Learner):
    def __init__(self, initial):
        super().__init__(initial)
        self.carried = False
        self.destination = None
        self.items = []
        self.finished_items = set()
        self.failed_items = set()
        self.current_item = None
        self.pending_interaction = None
        self.deliveries = 0
        self.failed_drop_positions = set()
        self.last_direction = None

    def learn(self, before, after, action):
        old_counts = collections.Counter(v for row in before for v in row)
        new_counts = collections.Counter(v for row in after for v in row)
        super().learn(before, after, action)
        if action in (1, 2, 3, 4):
            self.last_direction = action
        if self.palette and self.destination is None:
            actor = self.actor(before)
            candidates = [obj for obj in self.initial
                          if actor is None or obj[4] != actor[4]]
            if candidates:
                self.destination = max(candidates, key=area)
                self.items = [obj for obj in candidates if obj != self.destination]

        if action != 5 or self.palette is None:
            return
        changes = [new_counts[c] - old_counts[c] for c in self.palette]
        gain, loss = max(changes), min(changes)
        minimum_change = max(3, len(self.palette) * 2)
        if not self.carried and gain >= minimum_change:
            self.carried = True
            if self.current_item is not None:
                self.finished_items.add(self.current_item)
            print("OBSERVED_PICKUP", self.current_item, "actor_color_gain", gain,
                  flush=True)
        elif self.carried and loss <= -minimum_change:
            self.carried = False
            self.deliveries += 1
            print("OBSERVED_DROP", self.deliveries, "actor_color_loss", loss,
                  flush=True)
            self.current_item = None
        elif self.pending_interaction == "pickup" and self.current_item is not None:
            self.failed_items.add(self.current_item)
            print("UNKNOWN_PICKUP", self.current_item, flush=True)
        elif self.pending_interaction == "drop":
            actor = self.actor(after)
            if actor is not None:
                center = base.pos(actor)
                self.failed_drop_positions.add((round(center[0]), round(center[1])))
            print("UNKNOWN_DROP", flush=True)
        self.pending_interaction = None

    def choose(self, grid, available):
        actions = [a for a in available if 1 <= a <= 5]
        unknown = [a for a in actions if a not in self.tried]
        if unknown or self.palette is None:
            return super().choose(grid, available)
        actor = self.actor(grid)
        if actor is None:
            return super().choose(grid, available)
        here = base.pos(actor)
        current = (round(here[0]), round(here[1]))

        if self.carried:
            goal = self.destination
            interaction_distance = 2
            self.pending_interaction = "drop"
        else:
            candidates = [obj for obj in self.items
                          if (obj[0], obj[1]) not in
                          self.finished_items | self.failed_items]
            if not candidates:
                return self.explore(grid, actions, here, current)
            goal = min(candidates, key=lambda obj: base.dist(here, base.pos(obj)))
            self.current_item = (goal[0], goal[1])
            interaction_distance = max(
                (abs(dx) + abs(dy) for dx, dy in self.vectors.values()),
                default=2,
            )
            self.pending_interaction = "pickup"

        if goal is None:
            return self.explore(grid, actions, here, current)
        gap = base.dist(here, base.pos(goal))
        if (gap <= interaction_distance and 5 in actions
                and not (self.carried and current in self.failed_drop_positions)):
            directed = [
                (base.dist((here[0] + dx, here[1] + dy), base.pos(goal)), a)
                for a, (dx, dy) in self.vectors.items()
                if a in (1, 2, 3, 4) and (dx, dy) != (0, 0)
            ]
            if directed:
                desired = min(directed)[1]
                if self.last_direction != desired:
                    self.action_at[(current[0], current[1], desired)] += 1
                    return desired
            return 5

        moves = []
        for action in actions:
            dx, dy = self.vectors.get(action, (0, 0))
            if (dx, dy) == (0, 0):
                continue
            future = (here[0] + dx, here[1] + dy)
            visits = self.positions[(round(future[0]), round(future[1]))]
            repeats = self.action_at[(current[0], current[1], action)]
            moves.append((base.dist(future, base.pos(goal)) + 0.2 * visits
                          + 1.5 * repeats, action))
        if moves:
            _, action = min(moves)
            self.action_at[(current[0], current[1], action)] += 1
            return action
        return self.explore(grid, actions, here, current)

    def explore(self, grid, actions, here, current):
        frontier = []
        for action in actions:
            dx, dy = self.vectors.get(action, (0, 0))
            if (dx, dy) == (0, 0):
                continue
            x, y = round(here[0] + dx), round(here[1] + dy)
            if 0 <= x < len(grid[0]) and 0 <= y < len(grid) - 1:
                frontier.append((self.positions[(x, y)],
                                 self.action_at[(current[0], current[1], action)], action))
        if not frontier:
            return min(actions)
        action = min(frontier)[-1]
        self.action_at[(current[0], current[1], action)] += 1
        return action


base.Learner = CarryLearner

if __name__ == "__main__":
    base.main()
