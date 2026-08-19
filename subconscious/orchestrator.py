import os
import subprocess
import time
import random
import sys

# List of classic Project Gutenberg IDs to ingest
BOOK_QUEUE = [
    1342, # Pride and Prejudice
    84,   # Frankenstein
    1661, # Adventures of Sherlock Holmes
    2701, # Moby Dick
    11,   # Alice in Wonderland
    174,  # The Picture of Dorian Gray
    46,   # A Christmas Carol
]

MAX_CONCURRENT_CRAWLERS = 3

def run_orchestrator():
    print("🤖 [Orchestrator] Starting Auto-Pilot Mode...")
    print(f"🤖 [Orchestrator] Max Concurrent Crawlers: {MAX_CONCURRENT_CRAWLERS}")
    
    active_processes = []
    
    while BOOK_QUEUE or active_processes:
        # Clean up finished processes
        for p in active_processes[:]:
            if p.poll() is not None:
                print(f"🏁 [Orchestrator] Crawler finished (PID {p.pid})")
                active_processes.remove(p)
                
        # Start new crawlers if we have capacity and books left
        while len(active_processes) < MAX_CONCURRENT_CRAWLERS and BOOK_QUEUE:
            book_id = BOOK_QUEUE.pop(0)
            print(f"🚀 [Orchestrator] Launching Crawler for Book #{book_id}...")
            
            # Run the crawler as a subprocess
            cmd = [sys.executable, "-m", "subconscious.crawler", "--gutenberg", str(book_id), "--limit", "1000"]
            p = subprocess.Popen(cmd)
            active_processes.append(p)
            
            time.sleep(2) # Stagger startups slightly
            
        print(f"📊 [Orchestrator] Status: {len(active_processes)} Crawlers Running, {len(BOOK_QUEUE)} Books in Queue.")
        time.sleep(10) # Check every 10 seconds

if __name__ == "__main__":
    run_orchestrator()
