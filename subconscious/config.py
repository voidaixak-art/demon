import os
from pathlib import Path
from dotenv import load_dotenv

# Search for .env in current directory or subconscious directory
env_path = Path(__file__).parent / ".env"
if not env_path.exists():
    env_path = Path.cwd() / ".env"

load_dotenv(dotenv_path=env_path)

AXIOM_ENGINE_URL = os.getenv("AXIOM_ENGINE_URL", "http://localhost:8080").rstrip("/")
keys_str = os.getenv("GEMINI_API_KEYS", "")
GEMINI_API_KEYS = [k.strip() for k in keys_str.split(",") if k.strip()]
# Fallback to single key if provided
if not GEMINI_API_KEYS and os.getenv("GEMINI_API_KEY"):
    GEMINI_API_KEYS = [os.getenv("GEMINI_API_KEY")]

THINK_INTERVAL_SECONDS = float(os.getenv("THINK_INTERVAL_SECONDS", "4"))
SLEEP_CYCLE_EVERY_N_THOUGHTS = int(os.getenv("SLEEP_CYCLE_EVERY_N_THOUGHTS", "15"))
DREAM_TRIALS_PER_SLEEP = int(os.getenv("DREAM_TRIALS_PER_SLEEP", "100"))
DREAM_SIMILARITY_THRESHOLD = float(os.getenv("DREAM_SIMILARITY_THRESHOLD", "0.52"))
