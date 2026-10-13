#!/usr/bin/env python3
import collections
import http.cookiejar
import json
import os
import sys
import urllib.request


def objects(grid, palette=None):
    h, w = len(grid), len(grid[0])
    bg = collections.Counter(c for row in grid for c in row).most_common(1)[0][0]
    seen, result = set(), []
    for y in range(h - 1):
        for x in range(w):
            if (x, y) in seen or grid[y][x] == bg or (palette and grid[y][x] not in palette):
                continue
            todo, pixels = [(x, y)], []
            seen.add((x, y))
            while todo:
                px, py = todo.pop()
                pixels.append((px, py, grid[py][px]))
                for nx, ny in ((px + 1, py), (px - 1, py), (px, py + 1), (px, py - 1)):
                    if 0 <= nx < w and 0 <= ny < h - 1 and (nx, ny) not in seen:
                        if grid[ny][nx] != bg and (palette is None or grid[ny][nx] in palette):
                            seen.add((nx, ny))
                            todo.append((nx, ny))
            xs, ys = [p[0] for p in pixels], [p[1] for p in pixels]
            result.append((min(xs), min(ys), max(xs), max(ys),
                           tuple(sorted(collections.Counter(p[2] for p in pixels).items()))))
    return result


def pos(obj):
    return ((obj[0] + obj[2]) / 2, (obj[1] + obj[3]) / 2)


def dist(a, b):
    return abs(a[0] - b[0]) + abs(a[1] - b[1])


class Learner:
    def __init__(self, initial):
        self.initial = objects(initial)
        self.palette = None
        self.targets = []
        self.tried = set()
        self.vectors = {}
        self.visited = set()
        self.action_at = collections.Counter()
        self.positions = collections.Counter()
        self.interacted = set()

    def actor(self, grid):
        if self.palette is None:
            return None
        candidates = objects(grid, self.palette)
        return max(candidates, key=lambda obj: (obj[2] - obj[0] + 1) *
                   (obj[3] - obj[1] + 1), default=None)

    def learn(self, before, after, action):
        self.tried.add(action)
        if self.palette is None:
            moved = [(a, b) for a in objects(before) for b in objects(after)
                     if a[4] == b[4] and pos(a) != pos(b) and dist(pos(a), pos(b)) <= 16]
            if len(moved) == 1:
                old, new = moved[0]
                self.palette = {color for color, _ in old[4]}
                self.targets = [obj for obj in self.initial if obj != old]
                self.vectors[action] = (round(pos(new)[0] - pos(old)[0]),
                                        round(pos(new)[1] - pos(old)[1]))
        else:
            old, new = self.actor(before), self.actor(after)
            if old and new:
                delta = (round(pos(new)[0] - pos(old)[0]), round(pos(new)[1] - pos(old)[1]))
                if delta != (0, 0) or action not in self.vectors:
                    self.vectors[action] = delta
                self.positions[(round(pos(new)[0]), round(pos(new)[1]))] += 1

    def choose(self, grid, available):
        actions = [a for a in available if 1 <= a <= 5]
        if not actions:
            raise RuntimeError("No discrete keyboard action available")
        unknown = [a for a in actions if a not in self.tried]
        if unknown:
            return unknown[0]
        actor = self.actor(grid)
        if actor is None:
            return min(actions, key=lambda a: (self.action_at[(0, 0, a)], a))
        here = pos(actor)
        place = (round(here[0]), round(here[1]))
        for _ in range(len(self.targets)):
            remaining = [t for t in self.targets if (t[0], t[1]) not in self.visited]
            if not remaining:
                break
            goal = min(remaining, key=lambda t: dist(here, pos(t)))
            key = (goal[0], goal[1])
            gap = dist(here, pos(goal))
            if gap <= 2:
                if 5 in actions and key not in self.interacted:
                    self.interacted.add(key)
                    return 5
                self.visited.add(key)
                continue
            candidates = []
            for a in actions:
                dx, dy = self.vectors.get(a, (0, 0))
                if (dx, dy) == (0, 0):
                    continue
                future = (here[0] + dx, here[1] + dy)
                cost = dist(future, pos(goal))
                cost += 0.25 * self.positions[(round(future[0]), round(future[1]))]
                cost += 0.75 * self.action_at[(place[0], place[1], a)]
                candidates.append((cost, a))
            if candidates and min(candidates)[0] < gap + 4:
                a = min(candidates)[1]
                self.action_at[(place[0], place[1], a)] += 1
                return a
            self.visited.add(key)
        frontier = []
        for a in actions:
            dx, dy = self.vectors.get(a, (0, 0))
            if (dx, dy) != (0, 0):
                x, y = round(here[0] + dx), round(here[1] + dy)
                if 0 <= x < len(grid[0]) and 0 <= y < len(grid) - 1:
                    frontier.append((self.positions[(x, y)],
                                     self.action_at[(place[0], place[1], a)], a))
        a = min(frontier)[-1] if frontier else min(
            actions, key=lambda action: self.action_at[(0, 0, action)]
        )
        self.action_at[(place[0], place[1], a)] += 1
        return a


def main():
    base = os.getenv("ATHLESIA_ARC_BASE_URL", "http://127.0.0.1:8001").rstrip("/")
    key = os.getenv("ATHLESIA_ARC_API_KEY", "1234")
    opener = urllib.request.build_opener(
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar())
    )

    def call(route, body=None):
        data = json.dumps(body).encode() if body is not None else None
        req = urllib.request.Request(
            base + route, data=data,
            headers={"X-API-Key": key, "Content-Type": "application/json"}
        )
        with opener.open(req, timeout=60) as response:
            return json.load(response)

    matches = [
        g["game_id"] for g in call("/api/games")
        if g["game_id"] == "wa30" or g["game_id"].startswith("wa30-")
    ]
    if len(matches) != 1:
        raise RuntimeError(f"WA30 game resolution failed: {matches}")
    game = matches[0]
    card = call(
        "/api/scorecard/open",
        {"tags": ["athlesia", "example-learning"],
         "opaque": {"runner": "athlesia_example_agent_v2"}}
    )["card_id"]

    failure = None
    try:
        state = call("/api/cmd/RESET", {"game_id": game, "card_id": card})
        brain = Learner(state["frame"][-1])
        for step in range(1, 121):
            if state["state"] != "NOT_FINISHED":
                break
            before = state["frame"][-1]
            action = brain.choose(before, state["available_actions"])
            old_level = state["levels_completed"]
            state = call(
                f"/api/cmd/ACTION{action}",
                {"game_id": game, "guid": state["guid"]}
            )
            if state["levels_completed"] > old_level:
                print("LEVEL_COMPLETE", step, state["levels_completed"], flush=True)
                brain = Learner(state["frame"][-1])
            else:
                brain.learn(before, state["frame"][-1], action)
            if step <= 5 or step % 10 == 0:
                print(
                    "STEP", step, "ACTION", action,
                    "LEVELS", state["levels_completed"],
                    "VECTORS", brain.vectors, flush=True
                )
    except Exception as exc:
        failure = exc
        print("RUN_ERROR", repr(exc), file=sys.stderr)
    finally:
        score = call("/api/scorecard/close", {"card_id": card})
        with open("/tmp/athlesia_example_scorecard.json", "w") as out:
            json.dump(score, out, indent=2)
        count = score.get("total_levels_completed", 0)
        print(
            "EXTERNAL_ACCEPTANCE",
            "PASS" if count else "FAIL",
            "LEVELS", count,
            "SCORE", score.get("score"),
            flush=True
        )
    if failure:
        raise failure
    if count == 0:
        raise SystemExit(2)


if __name__ == "__main__":
    main()
