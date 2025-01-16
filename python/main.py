import requests
import time
from dataclasses import dataclass


@dataclass
class Word:
    word: str
    start: float|None
    end: float|None

@dataclass
class QueryResult:
    transcript_id: str
    words: list[Word]

def make_url(query: str, page: int=0, context: int=0):
    query = query.replace(" ", "%20")
    return f"http://127.0.0.1:8000/search?query={query}&page={page}&context={context}"

def search(url: str):
    start = time.time()
    res = requests.get(url)
    end = time.time()
    took = end - start
    print(f"In {took*1000:.2f}ms")
    data = res.json()
    # words = [Word(word['word'], word['start'], word['none']) for word in data['words']]
    # QueryResult(data['transcript_id'], words)
    for result in data:
        words = [word['word'] for word in result['words']]
        # print(" ".join(words) + "\n")

def main():
    for i in range(20):
        url = make_url("by this it is evident", i, 3)
        print(f"Page {i}")
        search(url)

if __name__ == "__main__":
    main()
