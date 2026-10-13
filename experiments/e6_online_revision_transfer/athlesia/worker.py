"""JSON-lines workers. Solver does not import environment, even transitively."""
from __future__ import annotations

import sys
import time

from .protocol import (MAX_BYTES, Observation, Trace, Json, array, boolean,
                       dumps, integer, loads, record, token)
from .runtime import constrain, peak_rss_bytes


def read() -> object:
    line = sys.stdin.buffer.readline(MAX_BYTES + 2)
    if not line or len(line) > MAX_BYTES + 1 or not line.endswith(b"\n"):
        raise ValueError("Missing/truncated/oversized protocol frame")
    return loads(line.decode("utf-8"))


def write(value: Json) -> None:
    print(dumps(value), flush=True)


class WirePort:
    def send_action(self, action_token: str) -> Observation:
        write({"kind": "action", "token": action_token})
        p = record(read(), {"kind", "observation"})
        if p["kind"] != "observation":
            raise ValueError("Expected observation")
        return Observation.from_wire(p["observation"])


def solver() -> None:
    from .learner import Agent
    limits = constrain()
    agent = Agent()
    while True:
        raw = read()
        if raw == {"command": "stop"}:
            return
        start_cpu, start_wall = time.process_time_ns(), time.perf_counter_ns()
        start_event = len(agent.events)
        try:
            if isinstance(raw, dict) and raw.get("command") == "fit":
                request = record(raw, {"command", "traces"})
                traces = tuple(Trace.from_wire(t) for t in array(request["traces"], 32))
                agent.fit(traces)
                payload: dict[str, Json] = {"trained": True}
            elif isinstance(raw, dict) and raw.get("command") == "predict":
                request = record(raw, {"command", "observation", "action"})
                observation = Observation.from_wire(request["observation"])
                prediction = agent.predict_public(observation, token(request["action"]))
                payload = {"prediction": {node: value for node, value in prediction}}
            elif isinstance(raw, dict) and raw.get("command") == "learn":
                request = record(raw, {"command", "trace"})
                payload = agent.observe_transition(Trace.from_wire(request["trace"]))
            elif isinstance(raw, dict) and raw.get("command") == "goal":
                request = record(raw, {"command", "target", "desired"})
                payload = agent.solve_public_goal(
                    WirePort(), token(request["target"]), boolean(request["desired"])
                )
            else:
                request = record(raw, {"command"})
                if request["command"] == "revise":
                    payload = agent.revise(WirePort())
                elif request["command"] == "transfer":
                    payload = agent.transfer(WirePort())
                else:
                    raise ValueError("Unknown command")
            ok = True
        except Exception as error:
            payload = {"error_type": type(error).__name__, "error": str(error)[:256]}
            ok = False
        metrics: dict[str, Json] = {**limits, "cpu_ns": time.process_time_ns() - start_cpu,
                                  "wall_ns": time.perf_counter_ns() - start_wall,
                                  "peak_rss_bytes": peak_rss_bytes(),
                                  "serialized_memory_bytes": len(agent.memory_bytes()),
                                  "memory_sha256": agent.memory_digest()}
        write({"kind": "result", "ok": ok, "payload": payload,
               "events": list(agent.events[start_event:]), "metrics": metrics})


def environment() -> None:
    # This branch and module are absent from the hardened solver image.
    from .environment import World, WorldConfig
    limits = constrain()
    config = record(read(), {"seed", "guards", "flags", "episode_index",
                             "opaque_auxiliary_actions"})
    world = World(WorldConfig(integer(config["seed"], 0, 2**63 - 1),
                             tuple(integer(i, 0, 5) for i in array(config["guards"], 6)),
                             tuple(boolean(f) for f in array(config["flags"], 6)),
                             integer(config["episode_index"], 0, 1_000_000),
                             boolean(config["opaque_auxiliary_actions"])))
    while True:
        raw = read()
        if raw == {"command": "stop"}:
            return
        try:
            request = record(raw, {"token"})
            obs = world.send_action(token(request["token"]))
            write({"observation": obs.to_wire(), "metrics": {**limits, "peak_rss_bytes": peak_rss_bytes()}})
        except Exception:
            # No internal exception strings, rule state, or seeds on this channel.
            write({"error": "ENVIRONMENT_PROTOCOL_ERROR"})


if __name__ == "__main__":
    if sys.argv[1:] == ["solver"]:
        solver()
    elif sys.argv[1:] == ["environment"]:
        environment()
    else:
        raise SystemExit("Choose solver or environment")
