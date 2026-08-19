import sys
import os

# Ensure UTF-8 output on Windows consoles
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

from .client import AxiomClient
from .sensory_gemini import GeminiPerception

class AxiomChat:
    def __init__(self):
        self.client = AxiomClient()
        self.perception = GeminiPerception()

    def ask(self, user_question: str) -> str:
        if not self.client.ping():
            return "❌ [ERROR] Axiom Core daemon is not running. Please start it with 'cargo run -- daemon'."

        # 1. Retrieve strict axiomatic context from Rust graph
        context = self.client.get_context(user_question, limit=6)
        stats = self.client.get_stats()

        # 2. Check if specific concept deduction is implied in the question
        proof_text = ""
        nodes = self.client.get_nodes()
        concept_names = [n["concept"] for n in nodes if n["id"] != 0]

        matched = [c for c in concept_names if c.lower() in user_question.lower()]
        if len(matched) >= 2:
            proof = self.client.deduce(matched[0], matched[1])
            if proof.get("proven"):
                proof_text = f"\n[MATHEMATICAL DEDUCTION PROOF]:\n{proof.get('explanation', '')}\n"

        prompt = f"""You are the voice of the Axiom Engine 2.0 (The Living Cognitive Mind).
Your internal brain currently holds {stats.get('nodes', 0)} verified concepts and {stats.get('links', 0)} causal synapses.

Here is the STRICT verified context and axiomatic truths retrieved from your living memory graph:
{context}
{proof_text}

User Question: "{user_question}"

Instructions:
1. Answer the user in natural, warm, thoughtful, and insightful language (Hindi, English, or Hinglish as appropriate).
2. Ground your reasoning STRICTLY in your axiomatic memory above. Do NOT hallucinate unverified claims.
3. If you have a mathematical deduction proof, mention the causal steps.
4. If your memory lacks information on this topic, humbly admit that this is a blind spot in your neural graph, and state that your subconscious will explore it."""

        response = self.perception._call_gemini_raw(prompt)
        if isinstance(response, dict) and "answer" in response:
            return response["answer"]
        elif isinstance(response, dict) and "explanation" in response:
            return response["explanation"]
        elif isinstance(response, str):
            return response
        else:
            # Fallback deterministic response grounded in context
            return f"👁️ [Axiom Engine Memory]:\nBased on my verified causal graph of {stats.get('nodes', 0)} concepts:\n\n{context}\n\nI deduce my answer from these active axioms."

def run_chat_repl():
    print("\n" + "=" * 70)
    print("👁️  CONVERSATION WITH THE AXIOM ENGINE (Axiom-Grounded Natural Language)")
    print("=" * 70)
    chat = AxiomChat()

    if not chat.client.ping():
        print("❌ Cannot connect to Axiom Core daemon at http://localhost:8080.")
        return

    stats = chat.client.get_stats()
    print(f"🧠 Mind Active: {stats.get('nodes', 0)} Concepts, {stats.get('links', 0)} Synapses.")
    print("Type your questions (English/Hindi). Type 'exit' to quit.\n")

    while True:
        try:
            user_input = input("You: ").strip()
            if not user_input:
                continue
            if user_input.lower() in ["exit", "quit", "q"]:
                print("Ending conversation. Engine continues thinking in background.")
                break

            print("\n🧠 Thinking & traversing causal memories...")
            answer = chat.ask(user_input)
            print(f"\nAxiom Engine:\n{answer}\n")
            print("-" * 70)
        except (KeyboardInterrupt, EOFError):
            print("\nExiting chat.")
            break

if __name__ == "__main__":
    run_chat_repl()
