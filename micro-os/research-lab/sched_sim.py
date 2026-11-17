#!/usr/bin/env python3
"""
==============================================================================
MicroOS Research Activity: Workload-Aware Scheduling
Paper: Can workload characteristics be used to select a more suitable scheduling strategy?
Candidate: Yeddula Vyshnavi (USN: 25BTRCL294, Section: E)
Institution: JAIN, FET CAMPUS (25PCCS201: Operating Systems)
==============================================================================
"""

import random
import statistics
import sys
import heapq
from collections import deque

CS_COST = 0.5  # context-switch cost charged on every switch between jobs
DEFAULT_QUANTUM = 10.0

def make_workload(kind, n, seed, load=0.8):
    """Generates synthetic job trace according to specified distribution."""
    rnd = random.Random(seed)
    if kind == "uniform":
        bursts = [rnd.uniform(8, 12) for _ in range(n)]
    elif kind == "short":
        bursts = [max(0.5, rnd.expovariate(1 / 5)) for _ in range(n)]
    elif kind == "bimodal":
        bursts = [rnd.expovariate(1 / 80) if rnd.random() < 0.2 else rnd.expovariate(1 / 4) for _ in range(n)]
    elif kind == "heavy":
        bursts = [min(500, 2 * rnd.paretovariate(1.5)) for _ in range(n)]
    elif kind == "batch":
        bursts = [max(0.5, rnd.expovariate(1 / 20)) for _ in range(n)]
    else:
        raise ValueError(f"Unknown workload kind: {kind}")

    t, jobs = 0.0, []
    mean_b = sum(bursts) / n
    for i, b in enumerate(bursts):
        if kind != "batch":
            t += rnd.expovariate(load / mean_b)
        jobs.append({"id": i, "arrival": 0.0 if kind == "batch" else t, "burst": b})
    return jobs

def features(jobs):
    """Extracts summary statistics: mean burst, CV, and arrival load factor."""
    b = [j["burst"] for j in jobs]
    m = statistics.mean(b)
    cv = statistics.pstdev(b) / m
    span = max(j["arrival"] for j in jobs) or 1.0
    load = sum(b) / span if span > 1.0 else float("inf")
    return {"mean_burst": m, "cv": cv, "load": load}

def simulate(jobs, policy, quantum=10.0):
    """Discrete-event simulation of FCFS, SJF, SRTF, and RR."""
    jobs = sorted(jobs, key=lambda j: (j["arrival"], j["id"]))
    n = len(jobs)
    rem = {j["id"]: j["burst"] for j in jobs}
    first, finish = {}, {}
    t, i, last, cs = 0.0, 0, None, 0
    ready = []  # min-heap for FCFS/SJF/SRTF
    rr = deque()

    def admit(upto):
        nonlocal i
        while i < n and jobs[i]["arrival"] <= upto:
            j = jobs[i]
            i += 1
            if policy == "RR":
                rr.append(j["id"])
            elif policy == "FCFS":
                heapq.heappush(ready, (j["arrival"], j["id"]))
            elif policy == "SJF":
                heapq.heappush(ready, (j["burst"], j["id"]))
            else:  # SRTF
                heapq.heappush(ready, (rem[j["id"]], j["id"]))

    done = 0
    while done < n:
        admit(t)
        if (policy == "RR" and not rr) or (policy != "RR" and not ready):
            if i < n:
                t = jobs[i]["arrival"]
                continue
            else:
                break

        if policy == "RR":
            jid = rr.popleft()
        else:
            jid = heapq.heappop(ready)[1]

        if last is not None and last != jid:
            t += CS_COST
            cs += 1
        last = jid
        first.setdefault(jid, t)

        if policy in ("FCFS", "SJF"):
            t += rem[jid]
            rem[jid] = 0
        elif policy == "RR":
            run = min(quantum, rem[jid])
            t += run
            rem[jid] -= run
            admit(t)
            if rem[jid] > 1e-12:
                rr.append(jid)
        else:  # SRTF: run until finish or next arrival that could preempt
            nxt = jobs[i]["arrival"] if i < n else float("inf")
            run = nxt - t if (nxt > t and nxt - t < rem[jid]) else rem[jid]
            t += run
            rem[jid] -= run
            admit(t)
            if rem[jid] > 1e-12:
                heapq.heappush(ready, (rem[jid], jid))

        if rem[jid] <= 1e-12 and jid not in finish:
            finish[jid] = t
            done += 1

    arr = {j["id"]: j["arrival"] for j in jobs}
    bur = {j["id"]: j["burst"] for j in jobs}
    tat = [finish[k] - arr[k] for k in arr]
    wait = [finish[k] - arr[k] - bur[k] for k in arr]
    resp = [first[k] - arr[k] for k in arr]

    return {
        "wait": statistics.mean(wait),
        "tat": statistics.mean(tat),
        "resp": statistics.mean(resp),
        "cs": cs
    }

POLICIES = ["FCFS", "SJF", "SRTF", "RR"]

def select_policy(f):
    """Rule-based, workload-aware selector (thresholds fixed before evaluation)."""
    # Rule 1: CV < 0.5 -> FCFS
    if f["cv"] < 0.5:
        return "FCFS"
    # Rule 2: Else, all jobs arrive together and CV < 1.5 -> SJF
    if f["cv"] < 1.5 and f["load"] == float("inf"):
        return "SJF"
    # Rule 3: Otherwise -> SRTF
    return "SRTF"

def run_experiment(N=200, RUNS=30):
    """Runs the full academic benchmark across 5 workloads and prints Tables 1-6."""
    kinds = ["uniform", "short", "bimodal", "heavy", "batch"]
    out = {}

    print("=" * 78)
    print(" 🔬 MicroOS Research Benchmark - Workload-Aware CPU Scheduling")
    print(" Author: Yeddula Vyshnavi | USN: 25BTRCL294 | Section: E")
    print(" Course: 25PCCS201 - Operating Systems Research Activity")
    print(f" Settings: {N} jobs/run | {RUNS} runs per workload (seeds 1000..{1000+RUNS-1})")
    print("=" * 78)

    for k in kinds:
        agg = {p: {"wait": [], "tat": [], "resp": [], "cs": []} for p in POLICIES + ["WA"]}
        chosen = {}
        features_list = []

        for r in range(RUNS):
            jobs = make_workload(k, N, seed=1000 + r)
            f = features(jobs)
            features_list.append(f)
            sel = select_policy(f)
            chosen[sel] = chosen.get(sel, 0) + 1

            res = {p: simulate(jobs, p) for p in POLICIES}
            res["WA"] = res[sel]

            for p in res:
                for m in res[p]:
                    agg[p][m].append(res[p][m])

        out[k] = (agg, chosen, features_list)

    # Print Table 1: Workload Features & Selector Choices
    print("\n" + "-" * 78)
    print(" TABLE 1: Workload Features and Selector Choice")
    print("-" * 78)
    print(f"{'Workload':<12} {'Mean Burst':<12} {'CV of Bursts':<14} {'Selector Choice'}")
    print("-" * 78)
    for k in kinds:
        agg, chosen, fs = out[k]
        m_burst = statistics.mean([x["mean_burst"] for x in fs])
        m_cv = statistics.mean([x["cv"] for x in fs])
        ch_str = ", ".join([f"{pol} ({cnt}/{RUNS})" for pol, cnt in chosen.items()])
        print(f"{k:<12} {m_burst:<12.2f} {m_cv:<14.2f} {ch_str}")

    # Print Table 2: Average Waiting Time
    print("\n" + "-" * 78)
    print(" TABLE 2: Average Waiting Time (lower is better)")
    print("-" * 78)
    print(f"{'Workload':<12} {'FCFS':<10} {'SJF':<10} {'SRTF':<10} {'RR':<10} {'WA Selector':<12}")
    print("-" * 78)
    for k in kinds:
        agg, _, _ = out[k]
        w = {p: statistics.mean(agg[p]["wait"]) for p in POLICIES + ["WA"]}
        print(f"{k:<12} {w['FCFS']:<10.2f} {w['SJF']:<10.2f} {w['SRTF']:<10.2f} {w['RR']:<10.2f} {w['WA']:<12.2f}")

    # Print Table 3: Waiting-Time Reduction
    print("\n" + "-" * 78)
    print(" TABLE 3: Waiting-time Reduction from Workload-Aware Selection (%)")
    print("-" * 78)
    print(f"{'Workload':<12} {'vs. FCFS (%)':<25} {'vs. RR (%)':<25}")
    print("-" * 78)
    for k in kinds:
        agg, _, _ = out[k]
        w = {p: statistics.mean(agg[p]["wait"]) for p in POLICIES + ["WA"]}
        red_fcfs = ((w["FCFS"] - w["WA"]) / w["FCFS"]) * 100.0
        red_rr = ((w["RR"] - w["WA"]) / w["RR"]) * 100.0
        print(f"{k:<12} {red_fcfs:<25.1f}% {red_rr:<25.1f}%")

    # Print Table 4: Average Response Time
    print("\n" + "-" * 78)
    print(" TABLE 4: Average Response Time (lower is better)")
    print("-" * 78)
    print(f"{'Workload':<12} {'FCFS':<10} {'SJF':<10} {'SRTF':<10} {'RR':<10} {'WA Selector':<12}")
    print("-" * 78)
    for k in kinds:
        agg, _, _ = out[k]
        r = {p: statistics.mean(agg[p]["resp"]) for p in POLICIES + ["WA"]}
        print(f"{k:<12} {r['FCFS']:<10.2f} {r['SJF']:<10.2f} {r['SRTF']:<10.2f} {r['RR']:<10.2f} {r['WA']:<12.2f}")

    # Print Table 5: Average Context Switches
    print("\n" + "-" * 78)
    print(" TABLE 5: Average Context Switches per Run (200 jobs)")
    print("-" * 78)
    print(f"{'Workload':<12} {'FCFS':<10} {'SJF':<10} {'SRTF':<10} {'RR':<10} {'WA Selector':<12}")
    print("-" * 78)
    for k in kinds:
        agg, _, _ = out[k]
        c = {p: statistics.mean(agg[p]["cs"]) for p in POLICIES + ["WA"]}
        print(f"{k:<12} {c['FCFS']:<10.1f} {c['SJF']:<10.1f} {c['SRTF']:<10.1f} {c['RR']:<10.1f} {c['WA']:<12.1f}")

    # Print Table 6: Average Turnaround Time
    print("\n" + "-" * 78)
    print(" TABLE 6: Average Turnaround Time (lower is better)")
    print("-" * 78)
    print(f"{'Workload':<12} {'FCFS':<10} {'SJF':<10} {'SRTF':<10} {'RR':<10} {'WA Selector':<12}")
    print("-" * 78)
    for k in kinds:
        agg, _, _ = out[k]
        t = {p: statistics.mean(agg[p]["tat"]) for p in POLICIES + ["WA"]}
        print(f"{k:<12} {t['FCFS']:<10.2f} {t['SJF']:<10.2f} {t['SRTF']:<10.2f} {t['RR']:<10.2f} {t['WA']:<12.2f}")

    print("\n" + "=" * 78)
    print(" Benchmark execution completed successfully.")
    print("=" * 78 + "\n")

if __name__ == "__main__":
    run_experiment(200, 30)
