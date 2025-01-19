import requests
import time
from dataclasses import dataclass

def colorize(text, color) -> str:
    """
    Prints text to the terminal with a specified color using ANSI escape codes.

    Args:
        text: The string to print.
        color: The color to use. Supported colors are:
            "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
            "bright_black", "bright_red", "bright_green", "bright_yellow",
            "bright_blue", "bright_magenta", "bright_cyan", "bright_white"
    """

    color_codes = {
        "black": "\033[30m",
        "red": "\033[31m",
        "green": "\033[32m",
        "yellow": "\033[33m",
        "blue": "\033[34m",
        "magenta": "\033[35m",
        "cyan": "\033[36m",
        "white": "\033[37m",
        "bright_black": "\033[90m",
        "bright_red": "\033[91m",
        "bright_green": "\033[92m",
        "bright_yellow": "\033[93m",
        "bright_blue": "\033[94m",
        "bright_magenta": "\033[95m",
        "bright_cyan": "\033[96m",
        "bright_white": "\033[97m",
    }
    reset_code = "\033[0m"

    if color.lower() in color_codes:
        return f"{color_codes[color.lower()]}{text}{reset_code}"
    else:
        return f"Invalid color: {color}. Supported colors are: {', '.join(color_codes.keys())}"


@dataclass
class Word:
    word: str
    start: float|None
    end: float|None

@dataclass
class QueryResult:
    transcript_id: str
    words: list[Word]

def create_url(query: str, obj: dict[str, str]={}):
    query = query.replace(" ", "%20")
    optional_parameters: str = "&".join([f"{key}={value}" for key, value in obj.items()])
    return f"http://127.0.0.1:8000/search?query={query}&{optional_parameters}"

def search(url: str):
    start = time.time()
    res = requests.get(url)
    end = time.time()
    took = end - start
    print(f"In {took*1000:.2f}ms")
    data = res.json()
    for result in data[:10]:
        words = [colorize(word['word'], "red") if word["matched"] else word['word'] for word in result['words']]
        print(" ".join(words) + "\n")

def main():
    while True:
        query = input("Search: ")  # "by this it is evident"
        if query == "exit":
            break
        url = create_url(query, { "context": "20", "remove_stop_words": "false" })
        search(url)

if __name__ == "__main__":
    main()
