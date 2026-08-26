import os
import re
import requests
import time

class GutenbergCrawler:
    def __init__(self, engine_api_url="http://localhost:8080"):
        self.engine_api_url = engine_api_url
        self.cache_dir = "gutenberg_cache"
        os.makedirs(self.cache_dir, exist_ok=True)
        
        self.target_books = {
            "Alice in Wonderland": "https://www.gutenberg.org/files/11/11-0.txt",
            "English Grammar": "https://www.gutenberg.org/cache/epub/11463/pg11463.txt"
        }

    def fetch_book(self, title, url):
        cache_path = os.path.join(self.cache_dir, f"{title.replace(' ', '_')}.txt")
        if os.path.exists(cache_path):
            with open(cache_path, 'r', encoding='utf-8') as f:
                return f.read()
                
        print(f"Downloading {title} from Gutenberg...")
        try:
            response = requests.get(url, timeout=10)
            response.raise_for_status()
            text = response.text
            with open(cache_path, 'w', encoding='utf-8') as f:
                f.write(text)
            return text
        except Exception as e:
            print(f"Failed to download {title}: {e}")
            return ""

    def clean_and_split(self, text):
        if "*** START OF THE PROJECT GUTENBERG" in text:
            text = text.split("*** START OF THE PROJECT GUTENBERG")[1]
        if "*** END OF THE PROJECT GUTENBERG" in text:
            text = text.split("*** END OF THE PROJECT GUTENBERG")[0]
            
        sentences = re.split(r'(?<=[.!?]) +(?=[A-Z])', text)
        cleaned = []
        for s in sentences:
            s = s.replace('\n', ' ').replace('\r', '').strip()
            if len(s) > 10 and re.search(r'[a-zA-Z]', s):
                cleaned.append(s)
        return cleaned

    def feed_engine(self, sentence):
        try:
            print(f"[Language Feed] -> {sentence[:50]}...")
            response = requests.post(f"{self.engine_api_url}/api/learn_language", json={"sentence": sentence}, timeout=5)
            if response.status_code == 200:
                pass # Successfully fed to engine
            else:
                print(f"Failed to feed engine. Status code: {response.status_code}")
        except Exception as e:
            print(f"Error feeding engine: {e}")
            time.sleep(1) # Backoff if engine is not ready
        time.sleep(0.05)

    def run(self):
        for title, url in self.target_books.items():
            print(f"\nProcessing Book: {title}")
            text = self.fetch_book(title, url)
            if not text:
                continue
                
            sentences = self.clean_and_split(text)
            print(f"Extracted {len(sentences)} valid sentences for Tensor Contraction training.")
            
            for i, sentence in enumerate(sentences):
                self.feed_engine(sentence)
                
        print("\nLanguage Learning Batch Complete. DisCoCat tensors are forming in offline dreams.")

if __name__ == "__main__":
    crawler = GutenbergCrawler()
    crawler.run()
    crawler.run()
