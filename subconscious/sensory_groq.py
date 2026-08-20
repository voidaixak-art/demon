import json
import urllib.request
import urllib.error
from typing import Dict, Any, Optional
from .config import GROQ_API_KEY

class GroqPerception:
    def __init__(self, api_key: str = GROQ_API_KEY):
        self.api_key = api_key
        self.model_name = "llama3-8b-8192"

    def _call_gemini_raw(self, prompt: str, max_retries: int = 3) -> Optional[Dict[str, Any]]:
        # Using the same method name as sensory_gemini to minimize changes in chat.py
        import os
        if not self.api_key:
            self.api_key = os.getenv("GROQ_API_KEY", "")

        if not self.api_key or "YOUR_GROQ_API_KEY" in self.api_key:
            print(f"❌ [Groq] Missing GROQ_API_KEY. (Length seen: {len(self.api_key) if self.api_key else 0})")
            return None

        url = "https://api.groq.com/openai/v1/chat/completions"
        payload = {
            "model": self.model_name,
            "messages": [{"role": "user", "content": prompt}],
            "response_format": {"type": "json_object"}
        }

        try:
            req = urllib.request.Request(
                url,
                data=json.dumps(payload).encode("utf-8"),
                headers={
                    "Content-Type": "application/json",
                    "Authorization": f"Bearer {self.api_key}"
                }
            )
            with urllib.request.urlopen(req, timeout=10) as resp:
                data = json.loads(resp.read().decode("utf-8"))
                content = data["choices"][0]["message"]["content"]
                return json.loads(content)
        except urllib.error.HTTPError as e:
            print(f"🔴 [Groq] HTTP Error: {e.code}")
            try:
                err_data = e.read().decode('utf-8')
                print(err_data)
            except:
                pass
            return None
        except Exception as e:
            print(f"❌ [Groq] Error: {e}")
            return None
