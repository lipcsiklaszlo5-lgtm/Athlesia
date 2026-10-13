"""Trusted evaluator/broker. It alone knows seeds and private fixture setup.

Default: independent processes, strict API, no Python object sharing.
--docker-solver: networkless, read-only solver container without environment code.
The default subprocess mode is NOT a sandbox against a malicious submitted agent.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import random
import selectors
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from typing import cast

from .protocol import (MAX_BYTES, Json, Kind, Observation, Relation, Trace, OBSERVE,
                       boolean, dumps, loads, record)
from .runtime import constrain, peak_rss_bytes


class Process:
    def __init__(self, kind: str, docker_solver: bool = False) -> None:
        self.directory = tempfile.TemporaryDirectory(prefix=f"athlesia_{kind}_")
        root = Path(self.directory.name)
        package = root / "athlesia"
        package.mkdir()
        common = ("__init__.py", "protocol.py", "runtime.py", "worker.py")
        specific = ("learner.py", "graph.py") if kind == "solver" else ("environment.py",)
        for name in common + specific:
            shutil.copyfile(Path(__file__).parent / name, package / name)
        launcher = root / "launch.py"
        launcher.write_text("import sys,runpy\nfrom pathlib import Path\nsys.path.insert(0,str(Path(__file__).parent))\nrunpy.run_module('athlesia.worker',run_name='__main__')\n")
        env = {"PATH": os.defpath, "LANG": "C.UTF-8", "PYTHONHASHSEED": "0"}
        if docker_solver and kind == "solver":
            cpu = min(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else 0
            command = ["docker", "run", "--rm", "-i", "--network=none", "--read-only",
                       "--memory=64m", "--memory-swap=64m", "--pids-limit=16", "--cap-drop=ALL",
                       "--security-opt=no-new-privileges", "--user=65534:65534",
                       f"--cpuset-cpus={cpu}", "athlesia-day1-solver"]
        else:
            command = [sys.executable, "-I", "-B", str(launcher), kind]
        self.child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                      stderr=subprocess.PIPE, cwd=root, env=env, bufsize=0)
        if self.child.stdin is None or self.child.stdout is None:
            raise RuntimeError("Missing process pipes")
        self.input, self.output = self.child.stdin, self.child.stdout
        self.buffer = bytearray()

    def send(self, value: Json) -> None:
        data = (dumps(value) + "\n").encode()
        while data:
            size = self.input.write(data)
            if size is None or size <= 0:
                raise RuntimeError("Broken worker input")
            data = data[size:]
        self.input.flush()

    def receive(self, timeout: float = 10.0) -> object:
        deadline = time.monotonic() + timeout
        with selectors.DefaultSelector() as selector:
            selector.register(self.output, selectors.EVENT_READ)
            while b"\n" not in self.buffer:
                remaining = deadline - time.monotonic()
                if remaining <= 0 or not selector.select(remaining):
                    raise TimeoutError("Worker deadline exceeded")
                chunk = os.read(self.output.fileno(), 16_384)
                if not chunk:
                    raise RuntimeError(f"Worker exited unexpectedly ({self.child.poll()})")
                self.buffer.extend(chunk)
                if len(self.buffer) > MAX_BYTES + 1:
                    raise ValueError("Worker message exceeds cap")
        line, _, rest = self.buffer.partition(b"\n")
        self.buffer = bytearray(rest)
        return loads(line.decode())

    def close(self) -> None:
        try:
            if self.child.poll() is None:
                self.send({"command": "stop"})
                self.child.wait(timeout=2)
        except (OSError, RuntimeError, subprocess.TimeoutExpired):
            self.child.kill()
            self.child.wait(timeout=2)
        finally:
            for stream in (self.child.stdin, self.child.stdout, self.child.stderr):
                if stream:
                    stream.close()
            self.directory.cleanup()

    def __enter__(self) -> Process:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()


class RemoteWorld:
    def __init__(self, process: Process, seed: int, guards: tuple[int, ...],
                 flags: tuple[bool, ...], episode: int,
                 opaque_auxiliary_actions: bool = False) -> None:
        self.process = process
        self.process.send({"seed": seed, "guards": list(guards), "flags": list(flags),
                           "episode_index": episode,
                           "opaque_auxiliary_actions": opaque_auxiliary_actions})
        self.peak = 0
        self.calls = 0
        self.actions = 0
        self.task_attempts = 0
        self.last_observation: Observation | None = None
        self.transcript: list[Json] = []

    def send_action(self, action_token: str) -> Observation:
        self.process.send({"token": action_token})
        reply = record(self.process.receive(), {"observation", "metrics"})
        metrics = cast(dict[str, object], reply["metrics"])
        self.peak = max(self.peak, int(cast(int, metrics["peak_rss_bytes"])))
        observation = Observation.from_wire(reply["observation"])
        self.calls += 1
        self.actions += action_token != OBSERVE
        self.task_attempts += any(e.source == action_token and e.relation == Relation.ACTS_ON
                                  for e in observation.scene.edges)
        self.last_observation = observation
        self.transcript.append({"action_token": action_token, "observation": observation.to_wire()})
        return observation


def independent_success(world: RemoteWorld) -> bool:
    """Judge actual environment output, never the solver's claimed success."""
    obs = world.last_observation
    if obs is None or world.task_attempts != 1:
        return False
    goals = [n.id for n in obs.scene.nodes if n.kind == Kind.GOAL]
    return bool(goals) and all(obs.value(g) for g in goals)


def solver_command(solver: Process, command: dict[str, Json], world: RemoteWorld | None = None) -> dict[str, Json]:
    solver.send(command)
    steps = 0
    while True:
        response = solver.receive()
        if isinstance(response, dict) and response.get("kind") == "action":
            p = record(response, {"kind", "token"})
            if world is None or not isinstance(p["token"], str):
                raise RuntimeError("Unauthorized solver action")
            steps += 1
            if steps > 48:
                raise RuntimeError("Real action budget exhausted")
            observation = world.send_action(p["token"])
            solver.send({"kind": "observation", "observation": observation.to_wire()})
        else:
            p = record(response, {"kind", "ok", "payload", "events", "metrics"})
            if p["kind"] != "result":
                raise RuntimeError("Unexpected solver message")
            return cast(dict[str, Json], p)


def run_case(seed: int = 17, channels: int = 2, guards: tuple[int, ...] = (0,), docker_solver: bool = False) -> dict[str, Json]:
    start = time.perf_counter_ns()
    traces: list[Trace] = []
    environment_peak = 0
    environment_calls, environment_actions = 0, 0
    demonstrations_transcript: list[Json] = []

    def capture_demo(flags: tuple[bool, ...], episode: int,
                     selected: str | None = None) -> tuple[str, tuple[str, ...]]:
        nonlocal environment_peak, environment_calls, environment_actions
        with Process("environment") as process:
            world = RemoteWorld(process, seed, guards, flags, episode, True)
            before = world.send_action(OBSERVE)
            actions = tuple(sorted(e.source for e in before.scene.edges
                                   if e.relation == Relation.ACTS_ON))
            incoming = {edge.target for edge in before.scene.edges
                        if edge.relation == Relation.LINK}
            roots = {node.id for node in before.scene.nodes
                     if node.kind == Kind.OBJECT and node.id not in incoming}
            task_actions = [edge.source for edge in before.scene.edges
                            if edge.relation == Relation.ACTS_ON and edge.target in roots]
            action = selected if selected is not None else (
                task_actions[0] if len(task_actions) == 1 else None
            )
            if action is None or action not in actions:
                raise RuntimeError("Invalid training fixture")
            after = world.send_action(action)
            traces.append(Trace(before, action, after))
            environment_peak = max(environment_peak, world.peak)
            environment_calls += world.calls
            environment_actions += world.actions
            return action, actions

    # The first trace identifies the task action by demonstration order, not by
    # token meaning.  Two unrelated calibration actions are then interleaved as
    # distractors.  Each has a changing and an idempotent witness.
    primary, observed_actions = capture_demo((True,) * channels, 0)
    for i in range(1, 3):
        capture_demo((True,) * channels, i, primary)
    auxiliary = tuple(action for action in observed_actions if action != primary)
    if len(auxiliary) != 2:
        raise RuntimeError("Expected two opaque auxiliary actions")
    episode = 10
    for action in auxiliary:
        capture_demo((False,) * channels, episode, action)
        capture_demo((True,) * channels, episode + 1, action)
        episode += 2
    tail = traces[1:]
    random.Random(seed ^ 0xA71E51A).shuffle(tail)
    traces[:] = [traces[0], *tail]
    demonstrations_transcript = [
        event
        for trace in traces
        for event in (
            {"action_token": OBSERVE, "observation": trace.before.to_wire()},
            {"action_token": trace.action, "observation": trace.after.to_wire()},
        )
    ]
    with Process("solver", docker_solver) as solver:
        fit = solver_command(solver, {"command": "fit", "traces": [t.to_wire() for t in traces]})
        schema_checks: list[dict[str, Json]] = []
        schema_validation_transcript: list[Json] = []
        prediction_results: list[dict[str, Json]] = []
        online_results: list[dict[str, Json]] = []
        online_checks: list[dict[str, Json]] = []
        patterns = (tuple(i % 2 == 0 for i in range(channels)),
                    tuple(i % 2 == 1 for i in range(channels)))
        validation_episode = 100
        for action in auxiliary:
            for flags in patterns:
                with Process("environment") as process:
                    held_out = RemoteWorld(process, seed, guards, flags,
                                           validation_episode, True)
                    before = held_out.send_action(OBSERVE)
                    prediction = solver_command(
                        solver,
                        {"command": "predict", "observation": before.to_wire(),
                         "action": action},
                    )
                    prediction_results.append(prediction)
                    payload = record(prediction["payload"], {"prediction"})
                    predicted_raw = record(payload["prediction"],
                                           {node for node, _ in before.values})
                    predicted = {node: boolean(value)
                                 for node, value in predicted_raw.items()}
                    after = held_out.send_action(action)
                    actual = dict(after.values)
                    exact = predicted == actual
                    online = solver_command(
                        solver, {"command": "learn",
                                 "trace": Trace(before, action, after).to_wire()})
                    online_results.append(online)
                    outcome = online.get("payload")
                    accepted = (online["ok"] is True and isinstance(outcome, dict)
                                and outcome.get("status") == "validated"
                                and outcome.get("contradiction") is False
                                and outcome.get("local_commit") is False)
                    online_checks.append({"action_token": action,
                                          "online_validation_accepted": accepted})
                    schema_checks.append({"action_token": action,
                                          "held_out_flags": list(flags),
                                          "predicted_values": predicted,
                                          "actual_values": actual,
                                          "exact_successor_prediction": exact})
                    schema_validation_transcript.extend(held_out.transcript)
                    environment_peak = max(environment_peak, held_out.peak)
                    environment_calls += held_out.calls
                    environment_actions += held_out.actions
                validation_episode += 1
        schema_prediction_success = all(
            check["exact_successor_prediction"] is True for check in schema_checks
        )
        with Process("environment") as process:
            trap = RemoteWorld(process, seed, guards, (False,) * channels, 3, True)
            revision = solver_command(solver, {"command": "revise"}, trap)
            environment_peak = max(environment_peak, trap.peak)
            environment_calls += trap.calls
            environment_actions += trap.actions
        with Process("environment") as process:
            # Entirely new object and action tokens plus randomized grid layout.
            target = RemoteWorld(process, seed + 1_000_003, guards,
                                 (False,) * channels, 4, True)
            transfer = solver_command(solver, {"command": "transfer"}, target)
            environment_peak = max(environment_peak, target.peak)
            environment_calls += target.calls
            environment_actions += target.actions
            evaluator_success = independent_success(target)
        results = (fit, *prediction_results, *online_results, revision, transfer)
        metrics = [cast(dict[str, Json], r["metrics"]) for r in results]
        peak = max(cast(int, m["peak_rss_bytes"]) for m in metrics)
        cpu = sum(cast(int, m["cpu_ns"]) for m in metrics)
        events = [event for r in results for event in cast(list[Json], r["events"])]
        fit_event_count = len(cast(list[Json], fit["events"]))
        events.insert(fit_event_count, {
            "event": "ACTION_SCHEMAS_HELD_OUT_VERIFIED",
            "source": "independent_broker",
            "predictions": len(schema_checks),
            "exact_predictions": sum(
                check["exact_successor_prediction"] is True for check in schema_checks
            ),
        })
        return {"seed": seed, "channels": channels, "hidden_guard_count_evaluator_only": len(guards),
                "passed": (all(r["ok"] is True for r in results)
                           and schema_prediction_success and evaluator_success
                           and all(x["online_validation_accepted"] is True for x in online_checks)),
                "independent_evaluator_success": evaluator_success,
                "action_schema_predictions_passed": schema_prediction_success,
                "action_schema_checks": schema_checks,
                "online_transition_checks": online_checks,
                "fit": fit, "revision": revision, "transfer": transfer, "events": events,
                "solver_cpu_ns": cpu, "case_wall_ns_including_startup": time.perf_counter_ns() - start,
                "solver_peak_rss_bytes": peak, "environment_peak_rss_bytes": environment_peak,
                "broker_peak_rss_bytes": peak_rss_bytes(),
                "sum_of_process_peak_rss_upper_bound": peak + environment_peak + peak_rss_bytes(),
                "environment_calls": environment_calls, "environment_actions": environment_actions,
                "public_transcripts": {"demonstrations": demonstrations_transcript,
                                       "action_schema_validation": schema_validation_transcript,
                                       "revision": trap.transcript, "transfer": target.transcript},
                "isolation": "docker_solver_network_none" if docker_solver else "trusted_code_subprocess_protocol_only"}


def percentile(values: list[int], q: float) -> int:
    return sorted(values)[max(0, math.ceil(len(values) * q) - 1)]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cases", type=int, default=1)
    parser.add_argument("--seed", type=int, default=17)
    parser.add_argument("--output", type=Path, default=Path("results/benchmark.json"))
    parser.add_argument("--docker-solver", action="store_true")
    args = parser.parse_args()
    if not 1 <= args.cases <= 10_000:
        parser.error("--cases must be in 1..10000")
    limits = constrain()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    run_log = args.output.with_suffix(".jsonl")
    aggregates: list[dict[str, Json]] = []
    source_hash = hashlib.sha256()
    for file in sorted(Path(__file__).parent.glob("*.py")):
        source_hash.update(file.name.encode() + b"\0" + file.read_bytes())
    with run_log.open("w") as output:
        for i in range(args.cases):
            channels = 3 + (i % 3)
            guards: tuple[int, ...] = (i % channels,) if i % 4 else (0,)
            if i and i % 5 == 0:
                guards = (0, channels - 1)
            result = run_case(args.seed + i * 7919, channels, guards, args.docker_solver)
            output.write(json.dumps(result, separators=(",", ":")) + "\n")
            output.flush()
            aggregates.append({k: result[k] for k in ("passed", "solver_cpu_ns", "case_wall_ns_including_startup",
                                                       "solver_peak_rss_bytes", "sum_of_process_peak_rss_upper_bound",
                                                       "action_schema_checks",
                                                       "online_transition_checks")})
            if args.cases == 1:
                for event in cast(list[Json], result["events"]):
                    print(json.dumps(event, ensure_ascii=False))
            elif (i + 1) % 10 == 0:
                print(f"COMPLETED {i + 1}/{args.cases}", flush=True)
    successes = sum(x["passed"] is True for x in aggregates)
    cpu_ns = [cast(int, x["solver_cpu_ns"]) for x in aggregates]
    wall_ns = [cast(int, x["case_wall_ns_including_startup"]) for x in aggregates]
    held_out_checks = [
        cast(dict[str, Json], check)
        for aggregate in aggregates
        for check in cast(list[Json], aggregate["action_schema_checks"])
    ]
    validated_online = sum(
        check["online_validation_accepted"] is True
        for aggregate in aggregates for check in aggregate["online_transition_checks"]
    )
    report = {"benchmark": "Athlesia bounded Day-3 online validation v0.3",
              "source_sha256": source_hash.hexdigest(), "python": sys.version.split()[0],
              "platform": platform.platform(), "processor": platform.processor(), **limits,
              "cases": args.cases, "successes": successes, "failures": args.cases - successes,
              "held_out_action_schema_predictions": len(held_out_checks),
              "exact_held_out_action_schema_predictions": sum(
                  check["exact_successor_prediction"] is True for check in held_out_checks
              ),
              "online_transition_validations": validated_online,
              "online_transition_validation_attempts": sum(
                  len(aggregate["online_transition_checks"]) for aggregate in aggregates),
              "solver_total_cpu_ms_p50": percentile(cpu_ns, 0.5) / 1e6,
              "solver_total_cpu_ms_p95": percentile(cpu_ns, 0.95) / 1e6,
              "case_wall_ms_p50_including_startup": percentile(wall_ns, 0.5) / 1e6,
              "case_wall_ms_p95_including_startup": percentile(wall_ns, 0.95) / 1e6,
              "max_solver_peak_rss_bytes": max(cast(int, x["solver_peak_rss_bytes"]) for x in aggregates),
              "max_sum_process_peak_rss_bytes": max(cast(int, x["sum_of_process_peak_rss_upper_bound"]) for x in aggregates),
              "isolation": "docker_solver_network_none" if args.docker_solver else "trusted_code_subprocess_protocol_only",
              "novelty_claim": False, "robotics_or_cross_mechanism_transfer_tested": False,
              "scope": "3..5 guard channels, three mixed opaque action schemas, fixed positive-conjunction grammar, relabelled isomorphic worlds"}
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    if successes != args.cases:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
