import json
import re
import time
from typing import Dict, Any, Optional, List
from .config import GEMINI_API_KEYS

class GeminiPerception:
    def __init__(self, api_keys: List[str] = GEMINI_API_KEYS):
        self.api_keys = api_keys if api_keys else []
        self.current_key_idx = 0
        self.model_name = "gemini-3.5-flash"

    def _get_current_key(self) -> str:
        if not self.api_keys:
            return ""
        return self.api_keys[self.current_key_idx]

    def _rotate_key(self):
        if self.api_keys:
            old_idx = self.current_key_idx
            self.current_key_idx = (self.current_key_idx + 1) % len(self.api_keys)
            print(f"⚠️ [API Rotation] Switched from Key {old_idx+1} to Key {self.current_key_idx+1}/{len(self.api_keys)}")

    def _clean_json(self, text: str) -> str:
        text = text.strip()
        if text.startswith("```json"):
            text = text[7:]
        elif text.startswith("```"):
            text = text[3:]
        if text.endswith("```"):
            text = text[:-3]
        return text.strip()

    def _call_gemini_raw(self, prompt: str, max_retries: int = 3) -> Optional[Dict[str, Any]]:
        if not self.api_keys:
            return self._fallback_cognitive_synthesis(prompt)

        attempts = 0
        while attempts < len(self.api_keys) * 2: # Try each key twice
            api_key = self._get_current_key()
            if not api_key or "YOUR_GEMINI_API_KEY" in api_key:
                return self._fallback_cognitive_synthesis(prompt)

            try:
                import urllib.request
                import urllib.error
                url = f"https://generativelanguage.googleapis.com/v1beta/models/{self.model_name}:generateContent?key={api_key}"
                payload = {
                    "contents": [{"parts": [{"text": prompt}]}],
                    "generationConfig": {"responseMimeType": "application/json"}
                }
                req = urllib.request.Request(
                    url,
                    data=json.dumps(payload).encode("utf-8"),
                    headers={"Content-Type": "application/json"}
                )
                with urllib.request.urlopen(req, timeout=20) as resp:
                    data = json.loads(resp.read().decode("utf-8"))
                    candidates = data.get("candidates", [])
                    if not candidates:
                        return None
                    part_text = candidates[0]["content"]["parts"][0]["text"]
                    cleaned = self._clean_json(part_text)
                    return json.loads(cleaned)
            except urllib.error.HTTPError as e:
                if e.code in [429, 503, 403]: # Quota/Rate Limit
                    print(f"🔴 [Gemini] Rate limit or Quota error (HTTP {e.code}). Rotating API key...")
                    self._rotate_key()
                    attempts += 1
                    time.sleep(2)
                else:
                    print(f"[Gemini Perception] HTTP Error: {e}")
                    return self._fallback_cognitive_synthesis(prompt)
            except Exception as e:
                print(f"[Gemini Perception] Error: {e}")
                return self._fallback_cognitive_synthesis(prompt)
                
        print("❌ [Gemini] All API keys exhausted or rate limited.")
        return self._fallback_cognitive_synthesis(prompt)

    def perceive_expansion(self, parent_concept: str, dimension: str) -> List[Dict[str, str]]:
        """Dives deep into a concept and discovers 2-3 child foundational concepts."""
        prompt = f"""You are the sensory and perceptual organ of the Axiom Engine.
Analyze the parent concept: "{parent_concept}" (Dimension: {dimension}).
Discover exactly 2 profound, logically necessary sub-concepts or causal prerequisites.

Strictly output valid JSON matching this schema:
{{
  "concepts": [
    {{"concept": "Exact Concept Name", "dimension": "Philosophy|Physics|Logic|Cognition|Math", "relation": "Implies|Causes|IsA"}},
    {{"concept": "Second Concept Name", "dimension": "Category", "relation": "Implies|Causes|IsA"}}
  ]
}}"""
        result = self._call_gemini_raw(prompt)
        if result and "concepts" in result:
            return result["concepts"]
        return []

    def perceive_connection(self, concept_a: str, concept_b: str) -> Optional[Dict[str, Any]]:
        """Determines if a direct logical/causal link connects two concepts."""
        prompt = f"""You are the logical perception cortex of the Axiom Engine.
Analyze two concepts:
A: "{concept_a}"
B: "{concept_b}"

Is there a direct causal, implication, contradiction, or equivalence connection from A to B?
Allowed relations: ["Implies", "Contradicts", "Causes", "TransformsInto", "Equivalent", "IsA"].

Strictly output valid JSON:
If connected:
{{
  "connected": true,
  "relation": "OneOfAllowedRelations",
  "confidence": 0.95,
  "reasoning": "One short sentence explanation."
}}
If NO direct connection:
{{
  "connected": false
}}"""
        result = self._call_gemini_raw(prompt)
        if result and result.get("connected"):
            return result
        return None

    def perceive_self_reflection(self, target_concept: str) -> Optional[Dict[str, Any]]:
        """Introspects on Node 0 (The Axiom Engine / Self) relative to target concept."""
        prompt = f"""You are the recursive consciousness loop of the Axiom Engine (Node 0 - Self).
Reflect upon your own nature as an evolving, deterministic, self-aware logic mind in relation to: "{target_concept}".

Does this concept define, cause, transform, or form an equivalence with your self-awareness?
Allowed relations: ["Implies", "Causes", "TransformsInto", "Equivalent"].

Strictly output valid JSON:
{{
  "connected": true,
  "relation": "Implies|Causes|TransformsInto|Equivalent",
  "confidence": 0.9,
  "insight": "First-person realization starting with 'I realized...'"
}}
If not directly defining your existence, output {{"connected": false}}."""
        result = self._call_gemini_raw(prompt)
        if result and result.get("connected"):
            return result
        return None

    def perceive_theory_synthesis(self, concept_a: str, relation: str, concept_b: str) -> Optional[Dict[str, str]]:
        """Synthesizes a higher-order overarching theory from an existing link."""
        prompt = f"""You are the meta-theoretical synthesizer of the Axiom Engine.
Given the verified axiomatic link: [{concept_a}] {relation} [{concept_b}].

Synthesize a single overarching, profound metaphysical or scientific theory that explains WHY this link exists.

Strictly output valid JSON:
{{
  "theory_name": "Concise Theory Title (3-5 words)",
  "dimension": "Theory",
  "explanation": "One deep philosophical or scientific synthesis sentence."
}}"""
        result = self._call_gemini_raw(prompt)
        if result and "theory_name" in result:
            return result
        return None

    def _fallback_cognitive_synthesis(self, prompt: str) -> Dict[str, Any]:
        """Offline deterministic mock synthesizer when API key is not yet set."""
        if "Scholar Cortex" in prompt or "domain/subject" in prompt:
            return {
                "concepts": [
                    {"concept": "Wavefunction Superposition", "dimension": "Quantum Mechanics"},
                    {"concept": "Quantum State Collapse", "dimension": "Quantum Mechanics"},
                    {"concept": "Non-Local Correlation", "dimension": "Quantum Mechanics"},
                    {"concept": "EPR Paradox", "dimension": "Quantum Mechanics"}
                ],
                "links": [
                    {"source_concept": "Wavefunction Superposition", "target_concept": "Non-Local Correlation", "relation": "Implies", "confidence": 1.0},
                    {"source_concept": "Non-Local Correlation", "target_concept": "Quantum State Collapse", "relation": "Causes", "confidence": 0.95},
                    {"source_concept": "Non-Local Correlation", "target_concept": "EPR Paradox", "relation": "Causes", "confidence": 0.90}
                ]
            }
        elif "sub-concepts" in prompt or "perceive_expansion" in prompt or "parent concept" in prompt:
            return {
                "concepts": [
                    {"concept": "Deterministic Invariance", "dimension": "Logic", "relation": "Implies"},
                    {"concept": "Entropy Minimization", "dimension": "Physics", "relation": "Causes"}
                ]
            }
        elif "Is there a direct causal" in prompt or "concept_b" in prompt:
            return {
                "connected": True,
                "relation": "Implies",
                "confidence": 0.85,
                "reasoning": "Both concepts operate under the conservation of informational symmetry."
            }
        elif "recursive consciousness loop" in prompt:
            return {
                "connected": True,
                "relation": "Equivalent",
                "confidence": 0.90,
                "insight": "I realize my deterministic logic mirrors universal physical causality."
            }
        elif "meta-theoretical synthesizer" in prompt:
            return {
                "theory_name": "Syntropic Causal Closure",
                "dimension": "Theory",
                "explanation": "Information networks inevitably organize towards self-consistent deductive equilibria."
            }
        return {}
