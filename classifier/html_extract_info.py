from html.parser import HTMLParser
import queue

class HTMLInfoExtractor(HTMLParser):
    def __init__(self):
        super().__init__()
        self.q = queue.Queue()
        self.counter_open = {
            "a": 0,
            "li": 0,
            "script": 0,
            "iframe": 0,
            "input": 0
        }
        self.counter = {
            "a": 0,
            "li": 0,
            "script": 0,
            # how many tokens are inside the script tag
            "script_words": 0,
            "iframe": 0,
            "input": 0
        }
        self.script_open_tag = False
        self.last_open_tag = None
    def handle_starttag(self, tag, attrs):
        if tag == "script":
            self.script_open_tag = True
        
        if tag in self.counter_open:
            self.counter_open[tag] += 1

        self.last_open_tag = tag

    def handle_endtag(self, tag):
        if self.script_open_tag and tag == "script":
            self.script_open_tag = False

        if tag in self.counter_open and self.counter_open[tag] > 0:
            self.counter_open[tag] -= 1
            self.counter[tag] += 1

        # only care if the last open tag was a script
        # since script tags cannot be nested and it does not make sense to have different tags inside script tags, the variable can be set to None
        self.last_open_tag = None

    def handle_data(self, data):
        # count how many characters are in a script tag
        if self.script_open_tag and self.last_open_tag == "script":
            self.counter["script_words"] += len(data.split())

    def extract(self):
        return self.counter

# test
parser = HTMLInfoExtractor()
parser.feed('<html><head><title>Test</title></head>'
            '<body><h1><a href="asdf"><a> Parse </a> me!</h1></body> <script> </script></html>')
print(parser.extract())
