#!/usr/bin/env python3
"""
Manual Snap Builder — Rebuilds axiom_brain.snap from WAL
Run this AFTER killing cargo run, BEFORE restarting it.

Usage:
  python rebuild_snap.py
"""
import json
import os
import sys
import time

SNAP_PATH = "axiom_brain.snap"
WAL_PATH = "axiom_brain.wal"
SNAP_OUT = "axiom_brain.snap"
SNAP_BACKUP = "axiom_brain.snap.bak"

def main():
    if not os.path.exists(WAL_PATH):
        print(f"ERROR: {WAL_PATH} not found!")
        return

    # Step 1: Load existing snap
    data = {
        "nodes": {},
        "links": [],
        "logs": [],
        "next_node_id": 1,
        "next_link_id": 1,
        "next_log_id": 1,
        "last_wal_line": 0,
    }

    if os.path.exists(SNAP_PATH):
        print(f"Loading existing snap: {SNAP_PATH}...")
        t0 = time.time()
        with open(SNAP_PATH, "r", encoding="utf-8") as f:
            data = json.load(f)
        elapsed = time.time() - t0
        print(f"  Snap loaded: {len(data.get('nodes', {}))} nodes, {len(data.get('links', []))} links ({elapsed:.1f}s)")
    else:
        print("No existing snap found, building from scratch.")

    skip_lines = data.get("last_wal_line", 0)
    print(f"  Will skip first {skip_lines} WAL lines (already in snap)")

    # Step 2: Convert links list to dict for O(1) dedup
    print("Converting links to fast dict...")
    links_dict = {}
    for link in data.get("links", []):
        rel = link.get("relation", "")
        if isinstance(rel, dict):
            rel_str = json.dumps(rel, sort_keys=True)
        else:
            rel_str = str(rel)
        key = (link["source_id"], link["target_id"], rel_str)
        links_dict[key] = link

    nodes = data.get("nodes", {})
    if nodes and isinstance(list(nodes.keys())[0], str):
        nodes = {int(k) if k.isdigit() else k: v for k, v in nodes.items()}

    logs = data.get("logs", [])
    next_node_id = data.get("next_node_id", 1)
    next_link_id = data.get("next_link_id", 1)
    next_log_id = data.get("next_log_id", 1)

    # Step 3: Replay WAL
    wal_size = os.path.getsize(WAL_PATH)
    print(f"\nReplaying WAL: {WAL_PATH} ({wal_size / (1024*1024*1024):.2f} GB)")

    t0 = time.time()
    line_count = 0
    processed = 0
    errors = 0

    with open(WAL_PATH, "r", encoding="utf-8") as f:
        for line in f:
            line_count += 1
            if line_count <= skip_lines:
                if line_count % 500_000 == 0:
                    print(f"  Skipping... {line_count}/{skip_lines} ({100*line_count/max(skip_lines,1):.0f}%)")
                continue

            processed += 1
            if processed % 100_000 == 0:
                elapsed = time.time() - t0
                print(f"  Processed {processed} new ops ({elapsed:.0f}s elapsed)...")

            trimmed = line.strip()
            if not trimmed:
                continue

            try:
                op = json.loads(trimmed)
            except json.JSONDecodeError:
                errors += 1
                continue

            if "InsertOrUpdateNode" in op:
                node = op["InsertOrUpdateNode"]
                nid = node["id"]
                str_key = str(nid)
                nodes[str_key] = node
                if nid >= next_node_id:
                    next_node_id = nid + 1

            elif "InsertLink" in op:
                link = op["InsertLink"]
                lid = link.get("id", 0)
                if lid >= next_link_id:
                    next_link_id = lid + 1
                rel = link.get("relation", "")
                if isinstance(rel, dict):
                    rel_str = json.dumps(rel, sort_keys=True)
                else:
                    rel_str = str(rel)
                key = (link["source_id"], link["target_id"], rel_str)
                links_dict[key] = link

            elif "DeleteLink" in op:
                dl = op["DeleteLink"]
                rel = dl.get("relation", "")
                if isinstance(rel, dict):
                    rel_str = json.dumps(rel, sort_keys=True)
                else:
                    rel_str = str(rel)
                key = (dl["source_id"], dl["target_id"], rel_str)
                links_dict.pop(key, None)

            elif "LogStream" in op:
                log = op["LogStream"]
                lid = log.get("id", 0)
                if lid >= next_log_id:
                    next_log_id = lid + 1
                logs.append(log)

    elapsed = time.time() - t0
    print(f"\nWAL replay done: {processed} new ops in {elapsed:.1f}s ({errors} parse errors)")

    # Step 4: Rebuild snap
    final_links = list(links_dict.values())
    snap_data = {
        "nodes": nodes,
        "links": final_links,
        "logs": logs,
        "next_node_id": next_node_id,
        "next_link_id": next_link_id,
        "next_log_id": next_log_id,
        "last_wal_line": line_count,
    }

    print(f"\nFinal: {len(nodes)} nodes, {len(final_links)} links, {len(logs)} logs")
    print(f"last_wal_line = {line_count}")

    # Step 5: Backup old snap
    if os.path.exists(SNAP_PATH):
        print(f"Backing up old snap to {SNAP_BACKUP}")
        if os.path.exists(SNAP_BACKUP):
            os.remove(SNAP_BACKUP)
        os.rename(SNAP_PATH, SNAP_BACKUP)

    # Step 6: Write new snap
    print(f"Writing fresh snap to {SNAP_OUT}...")
    t0 = time.time()
    with open(SNAP_OUT, "w", encoding="utf-8") as f:
        json.dump(snap_data, f)
    elapsed = time.time() - t0
    snap_size = os.path.getsize(SNAP_OUT)
    print(f"Done! Snap written: {snap_size / (1024*1024):.1f} MB in {elapsed:.1f}s")
    print(f"\n Next 'cargo run --release' will boot in ~2 seconds!")


if __name__ == "__main__":
    main()
