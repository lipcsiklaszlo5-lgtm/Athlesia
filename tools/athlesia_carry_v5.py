#!/usr/bin/env python3
"""Track the moving actor by its stable color, and verify interactions spatially."""
import collections

import athlesia_example_agent_v2 as base
from athlesia_carry_v3 import CarryLearner

ORIGINAL_LEARN = CarryLearner.__mro__[1].learn


def separation(a, b):
    """Manhattan distance between the nearest cells of two bounding boxes."""
    return max(0, b[0] - a[2], a[0] - b[2]) + max(
        0, b[1] - a[3], a[1] - b[3]
    )


class ContactLearner(CarryLearner):
    def __init__(self, initial):
        super().__init__(initial)
        self.first_frame = initial
        self.anchor_color = None
        self.actor_origin = None
        self.step = 0
        self.verified_transfers = 0
        self.failed_pickup_positions = set()

    def actor(self, grid):
        if not self.palette:
            return None
        if self.anchor_color is None:
            color_counts = collections.Counter(
                value for row in self.first_frame for value in row
            )
            self.anchor_color = max(self.palette, key=lambda c: color_counts[c])
        parts = base.objects(grid, {self.anchor_color})
        if not parts:
            return None
        if self.actor_origin is not None:
            return min(parts, key=lambda obj: base.dist(
                base.pos(obj), base.pos(self.actor_origin)
            ))
        return max(parts, key=lambda obj: (
            (obj[2] - obj[0] + 1) * (obj[3] - obj[1] + 1), -obj[1]
        ))

    def learn(self, before, after, action):
        self.step += 1
        ORIGINAL_LEARN(self, before, after, action)
        self.vectors.pop(5, None)
        previous_actor = self.actor(before)
        current_actor = self.actor(after)

        if (action in (1, 2, 3, 4) and previous_actor is not None
                and current_actor is not None
                and base.pos(previous_actor) == base.pos(current_actor)):
            here = base.pos(previous_actor)
            self.action_at[(round(here[0]), round(here[1]), action)] += 8
        if current_actor is not None:
            self.actor_origin = current_actor
        if action in (1, 2, 3, 4):
            self.last_direction = action

        if self.palette and self.destination is None:
            if previous_actor is None:
                raise RuntimeError("Moving sprite lost during initialization")
            origin = next(
                (obj for obj in self.initial if separation(obj, previous_actor) == 0),
                None,
            )
            targets = [obj for obj in self.initial if obj != origin]
            if targets:
                self.destination = max(targets, key=lambda obj: (
                    (obj[2] - obj[0] + 1) * (obj[3] - obj[1] + 1)
                ))
                self.items = [obj for obj in targets if obj != self.destination]

        if action != 5 or self.palette is None:
            if self.step % 10 == 0:
                print("TRACK", self.step, "ACTOR",
                      current_actor[:4] if current_actor else None,
                      "CARRY", self.carried, "ITEM", self.current_item,
                      flush=True)
            return

        counts_before = collections.Counter(x for row in before for x in row)
        counts_after = collections.Counter(x for row in after for x in row)
        first_counts = collections.Counter(
            x for row in self.first_frame for x in row
        )
        pigment = min(self.palette, key=lambda c: first_counts[c])
        change = counts_after[pigment] - counts_before[pigment]
        source = next(
            (obj for obj in self.items if (obj[0], obj[1]) == self.current_item),
            None,
        )
        close_to_source = (
            previous_actor is not None and source is not None
            and separation(previous_actor, source) <= 2
        )
        close_to_goal = (
            previous_actor is not None and self.destination is not None
            and separation(previous_actor, self.destination) <= 2
        )

        print("INTERACTION_LOCATION", self.step, "ACTOR",
              previous_actor[:4] if previous_actor else None,
              "SOURCE", self.current_item, "GOAL",
              self.destination[:4] if self.destination else None,
              flush=True)

        if not self.carried and change >= 8 and close_to_source:
            self.carried = True
            self.finished_items.add(self.current_item)
            print("PICKUP_CANDIDATE", self.step, self.current_item, flush=True)
        elif self.carried and change <= -8 and close_to_goal:
            self.carried = False
            self.verified_transfers += 1
            print("TRANSFER_CANDIDATE", self.step, self.verified_transfers,
                  "LEVEL_UNVERIFIED", flush=True)
            self.current_item = None
        else:
            if self.pending_interaction == "drop" and previous_actor is not None:
                center = base.pos(previous_actor)
                self.failed_drop_positions.add(
                    (round(center[0]), round(center[1]))
                )
            elif self.pending_interaction == "pickup" and close_to_source:
                self.failed_pickup_positions.add((self.current_item, previous_actor[:4]))
            print("UNCONFIRMED_INTERACTION", self.step, "pixel_delta", change,
                  "near_source", close_to_source, "near_goal", close_to_goal,
                  flush=True)
        self.pending_interaction = None

    def choose(self, grid, available):
        if self.step >= 35 and not self.finished_items:
            print("EARLY_STOP", "35 actions without a verified pickup", flush=True)
            raise SystemExit(3)
        known = all(a in self.tried for a in available if 1 <= a <= 5)
        if self.palette is not None and known:
            actor = self.actor(grid)
            if actor is not None and 5 in available:
                if self.carried and self.destination is not None:
                    center = base.pos(actor)
                    place = (round(center[0]), round(center[1]))
                    if (separation(actor, self.destination) <= 2
                            and place not in self.failed_drop_positions):
                        self.pending_interaction = "drop"
                        return 5
                elif not self.carried:
                    remaining = [obj for obj in self.items
                                 if (obj[0], obj[1]) not in
                                 self.finished_items | self.failed_items]
                    if remaining:
                        source = min(remaining, key=lambda obj:
                                     base.dist(base.pos(actor), base.pos(obj)))
                        self.current_item = (source[0], source[1])
                        x0, y0, x1, y1 = source[:4]
                        changed = sum(
                            grid[y][x] != self.first_frame[y][x]
                            for y in range(y0, y1 + 1)
                            for x in range(x0, x1 + 1)
                        )
                        pose = (self.current_item, actor[:4])
                        if separation(actor, source) <= 2 and changed >= 4:
                            if pose not in self.failed_pickup_positions:
                                self.pending_interaction = "pickup"
                                return 5
                            here = base.pos(actor)
                            choices = [
                                (
                                    base.dist(
                                        (here[0] + dx, here[1] + dy),
                                        base.pos(source)
                                    ) + 2 * self.action_at[
                                        (round(here[0]), round(here[1]), a)
                                    ],
                                    a
                                )
                                for a, (dx, dy) in self.vectors.items()
                                if a in available and a in (1, 2, 3, 4)
                                and (dx, dy) != (0, 0)
                            ]
                            if choices:
                                return min(choices)[1]
        return super().choose(grid, available)


base.Learner = ContactLearner

if __name__ == "__main__":
    base.main()
