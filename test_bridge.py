import time
import sys
import io

# Ensure UTF-8 output on Windows consoles
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

from subconscious.client import AxiomClient

def test_neuro_symbolic_integration():
    print("=" * 60)
    print("RUNNING END-TO-END NEURO-SYMBOLIC INTEGRATION TEST")
    print("=" * 60)

    client = AxiomClient()
    if not client.ping():
        print("[ERROR] Daemon is not running on port 8080. Start daemon first.")
        sys.exit(1)

    print("1. [OK] Pinged Axiom Core Daemon successfully.")

    # 1. Check Initial State
    stats = client.get_stats()
    print(f"2. [STATS] Initial: {stats['nodes']} Nodes, {stats['links']} Links, Density: {stats['density']}")
    assert stats['nodes'] >= 5, "Should have seeded Node 0 + 4 foundational axioms"

    # 2. Add New Verified Axioms
    res1 = client.add_node("Deterministic Necessity", "Philosophy")
    res2 = client.add_node("Universal Order", "Philosophy")
    print(f"3. [NODE] Added nodes: #{res1['id']} and #{res2['id']}")

    # 3. Form Synapses
    client.add_link(0, res1['id'], "Implies", 1.0)
    client.add_link(res1['id'], res2['id'], "Causes", 0.95)
    print("4. [SYNAPSE] Wired causal synapses: Self ==> Deterministic Necessity ==> Universal Order")

    # 4. Prove Deduction Path
    proof = client.deduce("The Axiom Engine (Self)", "Universal Order")
    print("\n5. [DEDUCTION PROOF RESULT]:")
    print(proof["explanation"])
    assert proof["proven"], "Must prove path from Self to Universal Order!"

    # 5. Run Dream Mutation
    dream = client.run_dream(trials=50, threshold=0.45)
    print(f"\n6. [DREAM REPORT]: {dream['total_trials']} trials, {len(dream['successful_synapses'])} new synapses formed.")

    # 6. Stream Log verification
    client.log_thought("TEST", "End-to-end neuro-symbolic verification passed with 100% certainty.")
    print("7. [LOG] Stream of consciousness logged successfully.")

    final_stats = client.get_stats()
    print(f"\n[SUCCESS] ALL INTEGRATION TESTS PASSED! Final Nodes: {final_stats['nodes']}, Synapses: {final_stats['links']}")
    print("=" * 60)

if __name__ == "__main__":
    test_neuro_symbolic_integration()
