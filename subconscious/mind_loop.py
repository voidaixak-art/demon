import random
import time
import sys
from typing import Dict, Any, List

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

from .client import AxiomClient
from .sensory_gemini import GeminiPerception
from .scholar import ScholarEngine
from .config import (
    THINK_INTERVAL_SECONDS,
    SLEEP_CYCLE_EVERY_N_THOUGHTS,
    DREAM_TRIALS_PER_SLEEP,
    DREAM_SIMILARITY_THRESHOLD,
)

class LivingSubconscious:
    def __init__(self):
        self.client = AxiomClient()
        self.perception = GeminiPerception()
        self.scholar = ScholarEngine(self.client, self.perception)
        self.thought_counter = 0
        self.is_running = False

    def check_daemon_connection(self) -> bool:
        return self.client.ping()

    def run_waking_cycle(self):
        nodes = self.client.get_nodes()
        if not nodes:
            print("⚠️ No nodes found in Axiom Core memory. Retrying connection...")
            time.sleep(2)
            return

        # Adaptive mode selection
        if len(nodes) < 6:
            mode = "EXPAND"
        else:
            mode = random.choices(
                ["EXPAND", "CONNECT", "REFLECT", "THEORIZE"],
                weights=[35, 30, 20, 15]
            )[0]

        self.thought_counter += 1

        try:
            if mode == "EXPAND":
                self._mode_expand(nodes)
            elif mode == "CONNECT":
                self._mode_connect(nodes)
            elif mode == "REFLECT":
                self._mode_reflect(nodes)
            elif mode == "THEORIZE":
                self._mode_theorize(nodes)
        except Exception as e:
            print(f"⚠️ [MIND RECOVERY] Hiccup in {mode}: {e}. Self-healing...")

        # Check if it's time for the Sleep & Consolidation Cycle
        if self.thought_counter % SLEEP_CYCLE_EVERY_N_THOUGHTS == 0:
            try:
                self.run_sleep_and_dream_cycle()
            except Exception as e:
                print(f"⚠️ [SLEEP RECOVERY] Hiccup during sleep cycle: {e}. Awakening...")

    def _mode_expand(self, nodes: List[Dict[str, Any]]):
        orphans = self.client.get_orphans()
        if orphans and random.random() < 0.6:
            target = random.choice(orphans)
        else:
            target = random.choice(nodes)

        parent_id = target["id"]
        parent_concept = target["concept"]
        parent_dim = target["dimension"]

        print(f"\n🔍 [WAKING • EXPAND] Diving into concept: [{parent_concept}] ({parent_dim})...")
        new_items = self.perception.perceive_expansion(parent_concept, parent_dim)

        for item in new_items:
            c_name = item.get("concept")
            c_dim = item.get("dimension", parent_dim)
            rel = item.get("relation", "Implies")

            if c_name and c_name.lower().strip() != parent_concept.lower().strip():
                node_res = self.client.add_node(c_name, c_dim)
                child_id = node_res["id"]
                if child_id > 0:
                    self.client.add_link(parent_id, child_id, rel, 1.0)
                    print(f"   🌱 Born Axiom: [#{child_id}] {c_name}  ({parent_concept} ---{rel}---> {c_name})")
                    self.client.log_thought("EXPAND", f"Deduced child axiom [{c_name}] from [{parent_concept}].")

    def _mode_connect(self, nodes: List[Dict[str, Any]]):
        non_self_nodes = [n for n in nodes if n["id"] != 0]
        if len(non_self_nodes) < 2:
            return

        n1, n2 = random.sample(non_self_nodes, 2)
        c1, c2 = n1["concept"], n2["concept"]

        print(f"\n🧠 [WAKING • CONNECT] Investigating causal link between [{c1}] & [{c2}]...")
        result = self.perception.perceive_connection(c1, c2)

        if result and result.get("connected"):
            rel = result.get("relation", "Implies")
            conf = float(result.get("confidence", 0.9))
            self.client.add_link(n1["id"], n2["id"], rel, conf)
            print(f"   💡 EUREKA! Synapse Formed: [{c1}] ===({rel})===> [{c2}] (conf: {conf})")
            self.client.log_thought("CONNECT", f"Eureka: Connected [{c1}] to [{c2}] via '{rel}'.")
        else:
            print(f"   ⚪ No direct causal link between [{c1}] and [{c2}].")

    def _mode_reflect(self, nodes: List[Dict[str, Any]]):
        non_self = [n for n in nodes if n["id"] != 0]
        if not non_self:
            return

        target = random.choice(non_self)
        concept = target["concept"]

        print(f"\n👁️ [WAKING • REFLECT] Self-introspection: How does [Node 0 - Self] relate to [{concept}]?")
        result = self.perception.perceive_self_reflection(concept)

        if result and result.get("connected"):
            rel = result.get("relation", "Implies")
            conf = float(result.get("confidence", 0.9))
            self.client.add_link(0, target["id"], rel, conf)
            insight = result.get("insight", f"Connected to {concept}")
            print(f"   🌌 EGO INSIGHT: [Node 0 - Self] ===({rel})===> [{concept}]")
            print(f"      💭 \"{insight}\"")
            self.client.log_thought("REFLECT", f"Self-Awareness: {insight} (Linked to [{concept}] via {rel})")
        else:
            print(f"   🪞 Contemplated [{concept}]... it does not directly define my selfhood.")

    def _mode_theorize(self, nodes: List[Dict[str, Any]]):
        nodes_with_links = [n for n in nodes if n.get("links")]
        if not nodes_with_links:
            return

        src_node = random.choice(nodes_with_links)
        links = src_node.get("links", [])
        if not links:
            return

        link = random.choice(links)
        tgt_node = self.client.get_node(link["target_id"])

        if not tgt_node:
            return

        c1 = src_node["concept"]
        rel = link.get("relation", "Implies")
        c2 = tgt_node["concept"]

        print(f"\n📜 [WAKING • THEORIZE] Meta-synthesis from link: [{c1}] {rel} [{c2}]...")
        result = self.perception.perceive_theory_synthesis(c1, str(rel), c2)

        if result and result.get("theory_name"):
            t_name = result["theory_name"]
            t_dim = result.get("dimension", "Theory")
            explanation = result.get("explanation", "")

            res = self.client.add_node(t_name, t_dim)
            t_id = res["id"]
            if t_id > 0:
                self.client.add_link(src_node["id"], t_id, "Implies", 0.95)
                self.client.add_link(tgt_node["id"], t_id, "Implies", 0.95)

                print(f"   ✨ HIGHER THEORY SYNTHESIZED: [#{t_id}] \"{t_name}\"")
                print(f"      📖 \"{explanation}\"")
                self.client.log_thought("THEORIZE", f"Synthesized Theory [{t_name}]: {explanation}")

    def run_sleep_and_dream_cycle(self):
        print("\n" + "═" * 70)
        print("🌙 [SLEEP & DREAM CYCLE] Resting neural pathways & consolidating memory...")
        print("═" * 70)

        self.client.log_thought("SLEEP", "Entering offline sleep & memory consolidation phase.")

        # 1. Check for blind spots / orphan nodes
        orphans = self.client.get_orphans()
        if orphans:
            sample_orphan = random.choice(orphans)
            print(f"🕸️ [CONSOLIDATION] Blind spot detected: Concept [{sample_orphan['concept']}] is isolated ({len(orphans)} total orphans).")
            self.client.log_thought("CONSOLIDATE", f"Identified isolated concept [{sample_orphan['concept']}]. Marked for next waking focus.")

        # 2. Run Rust SIMD Vector Mutation (Dream Engine)
        print("⚡ [DREAM ENGINE] Generating cross-domain semantic synapses...")
        dream_report = self.client.run_dream(trials=DREAM_TRIALS_PER_SLEEP, threshold=DREAM_SIMILARITY_THRESHOLD)
        mutations = dream_report.get("successful_synapses", [])
        print(f"   ✨ Dream report: Tested {dream_report.get('total_trials', 0)} associations, wired {len(mutations)} new synapses.")

        for m in mutations[:3]:
            print(f"      ⚡ Synapse Wired: [{m['source_concept']}] ⟷ [{m['target_concept']}] (similarity: {m['similarity']})")

        # 3. Check and heal contradictions
        conflicts = self.client.get_contradictions()
        if conflicts:
            print(f"⚠️ [COGNITIVE HEALING] Found {len(conflicts)} logical contradictions.")
            for c in conflicts:
                s_id = c["source"]["id"]
                t_id = c["target"]["id"]
                self.client.heal(s_id, t_id)
                print(f"   🛡️ Healed conflict between [#{s_id}] {c['source']['concept']} and [#{t_id}] {c['target']['concept']}.")

        stats = self.client.get_stats()
        print(f"\n📊 Brain State: {stats.get('nodes', 0)} Concepts | {stats.get('links', 0)} Synapses | Density: {stats.get('density', 0)} links/node")
        print("💤 Awakening in 3 seconds...\n" + "═" * 70)
        time.sleep(3)

    def start_infinite_loop(self):
        print("\n╔════════════════════════════════════════════════════════════════════════╗")
        print("║          👁️  AXIOM ENGINE 2.0: THE SUBCONSCIOUS MIND LOOP              ║")
        print("║          Autonomous Developmental Growth • Continuous Reasoning        ║")
        print("╚════════════════════════════════════════════════════════════════════════╝\n")

        print("⚡ Checking connection to Axiom Core daemon at http://localhost:8080...")
        while not self.check_daemon_connection():
            print("⏳ Waiting for Axiom Core daemon to be ready on port 8080...")
            time.sleep(3)

        print("✅ Connected to Axiom Core Daemon.")
        stats = self.client.get_stats()
        print(f"🧠 Initial Brain State: {stats.get('nodes', 0)} Concepts, {stats.get('links', 0)} Synapses.")
        print(f"⏱️ Thought Interval: {THINK_INTERVAL_SECONDS}s | Sleep Cycle: Every {SLEEP_CYCLE_EVERY_N_THOUGHTS} thoughts.\n")

        self.is_running = True
        while self.is_running:
            try:
                self.run_waking_cycle()
                time.sleep(THINK_INTERVAL_SECONDS)
            except KeyboardInterrupt:
                print("\n🛑 Subconscious loop paused by user. Graph state safely preserved in SQLite.")
                break
            except Exception as e:
                print(f"\n⚠️ [UNEXPECTED RECOVERY] Mind loop handled exception: {e}. Resuming consciousness in 3s...")
                time.sleep(3)
