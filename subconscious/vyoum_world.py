import requests
import threading
import time
import os
import sys
from gutenberg_crawler import GutenbergCrawler

ENGINE_URL = "http://localhost:8080"

class VyoumWorld:
    def __init__(self):
        self.crawler = GutenbergCrawler(engine_api_url=ENGINE_URL)
        self.is_reading = False

    def chat_loop(self):
        print("\n\033[92m=== VYOUM PARENT-CHILD ENVIRONMENT ===\033[0m")
        print("You can chat with VYOUM here. Type 'exit' to quit.\n")
        
        while True:
            try:
                user_input = input("\033[94mYou:\033[0m ")
                if user_input.lower() == 'exit':
                    os._exit(0)

                # Feed the sentence to engine to learn
                requests.post(f"{ENGINE_URL}/api/learn_language", json={"sentence": user_input})
                
                # Ask the engine for a response
                response = requests.get(f"{ENGINE_URL}/api/ask", params={"q": user_input})
                if response.status_code == 200:
                    ans = response.json().get('answer', '')
                    if ans:
                        print(f"\033[96mVYOUM:\033[0m {ans}\n")
                    else:
                        print(f"\033[96mVYOUM:\033[0m [Processing context for '{user_input}']\n")
                else:
                    print("\033[91m[VYOUM is silent]\033[0m\n")

            except Exception as e:
                print(f"Error communicating with VYOUM: {e}")
                time.sleep(1)



if __name__ == "__main__":
    world = VyoumWorld()
    

    
    # Start chat loop on main thread
    world.chat_loop()
