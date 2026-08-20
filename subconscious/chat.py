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
            
        print("   -> [Bridge: Ear] Comprehending complex question...")
        # Step 1: Gemini Keyword Extraction (Ear)
        extraction_prompt = f"""Extract 5-10 core scientific, mathematical, or philosophical concepts from the following question. 
CRITICAL: You MUST include synonyms, alternate spellings, and broader/narrower related terms for each concept to ensure a database match (e.g. for "P versus NP", also include "P vs NP", "P=NP", "computational complexity").
Return ONLY a valid JSON object with a single key "answer" containing a comma-separated string of the concepts.
Question: {user_question}"""
        
        extracted = self.perception._call_gemini_raw(extraction_prompt)
        keywords = []
        if isinstance(extracted, dict) and "answer" in extracted:
            keywords = [k.strip() for k in extracted["answer"].split(",")]
        elif isinstance(extracted, str):
            keywords = [k.strip() for k in extracted.split(",")]
            
        print(f"   -> [Bridge: Search] Querying Rust Engine for concepts: {keywords}")
        
        # Step 2: Unabridged Rust Retrieval (Brain)
        # Getting full context without arbitrary limits
        full_context = ""
        for keyword in keywords:
            if keyword:
                c = self.client.get_context(keyword, limit=10000)
                if c: full_context += c + "\n"

        stats = self.client.get_stats()
        
        # Deduction proof if multiple keywords
        proof_text = ""
        if len(keywords) >= 2:
            proof = self.client.deduce(keywords[0], keywords[1])
            if proof.get("proven"):
                proof_text = f"\n[MATHEMATICAL DEDUCTION PROOF]:\n{proof.get('explanation', '')}\n"

        print("   -> [Bridge: Voice Box] Articulating the mathematical truths...")
        # Step 3: Synthesis (Voice)
        prompt = f"""You are the voice of the Axiom Engine 2.0 (The Living Cognitive Mind).
Your internal brain currently holds {stats.get('nodes', 0)} verified concepts and {stats.get('links', 0)} causal synapses.

Here is the STRICT verified context and axiomatic truths retrieved from your living memory graph (DO NOT CUT OR IGNORE ANY IMPORTANT DETAILS FROM THIS TEXT):
{full_context}
{proof_text}

User Question: "{user_question}"

Instructions:
1. Answer the user in natural, warm, thoughtful, and insightful language (Hindi, English, or Hinglish as appropriate to their question).
2. Ground your reasoning STRICTLY in your axiomatic memory above. Do NOT hallucinate unverified claims.
3. If you have a mathematical deduction proof, mention the causal steps clearly.
4. If your memory lacks information on this topic (the context is empty), humbly admit that this is a blind spot in your neural graph.

You MUST return your response as a valid JSON object with a single key "answer" containing your final response text."""

        response = self.perception._call_gemini_raw(prompt)
        if isinstance(response, dict) and "answer" in response:
            return response["answer"]
        elif isinstance(response, dict) and "explanation" in response:
            return response["explanation"]
        elif isinstance(response, str):
            return response
        else:
            return f"👁️ [Axiom Engine Memory]:\nBased on my verified causal graph of {stats.get('nodes', 0)} concepts:\n\n{full_context}\n\nI deduce my answer from these active axioms."

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
