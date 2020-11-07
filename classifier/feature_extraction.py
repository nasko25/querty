#!/usr/bin/env python3

# These are the categories in the KI-04 dataset.
# The websites are sorted by their genre. Each type of website is put in a folder, the names of the folders are:
genres = {
    "articles",
    "discussion",
    "download",
    "help",
    "linklists",
    "portrait-non_priv",
    "portrait-priv",
    "shop"
}

'''
    In order to train the classifier I will use the following features:
        * the textual contents of the web page
            - use information gain to extract the features
        * the number of <a> tags
        * the number of <ol/ul>-<li> tags
        * number (and size? - number of words/characters) of <script> tags
        * number of <iframe> tags
        * number of <input> tags
        * <meta> tag values
        * ...
'''

import html2text
import os
from bs4 import BeautifulSoup

def extract_text(html):
    pass

data_dir = "data/genre-corpus-04"
data = {}

# inilitize empty lists for each genre in the data dictionary
for genre in genres:
    data[genre] = []

# save all html documents in a dictionary of lists
# for subdir, dirs, files in os.walk(data_dir):
#     dir = subdir.split("/")[-1]
#     if dir in genres:
#         for file_name in files:
#             file = open(subdir + "/" + file_name, 'r')
#             # TODO extract_text(file.read())
#             data[dir].append(file.read())
#             file.close()

random_file = "data/genre-corpus-04/articles/5440455052.html"

f = open(random_file, 'r')
html = f.read()
f.close()

# TODO will need to manually extract the text?
# TODO strip space characters (and \t\n)
h = html2text.HTML2Text()
h.ignore_links = True
# h.ignore_images = True ?
# print(h.handle(html))

soup = BeautifulSoup(html, features="html5lib")
print(' '.join(soup.get_text().split()))   # still has some scripts

print("\n\n")

metas = soup.find_all("meta")
# meta tags can have the following attributes:
#       - charset
#       - content
#       - http-equiv
#       - name
# print(metas)
# print(metas[0]["http-equiv"])
# print(metas[1]["content"])

meta_content = []
for meta in metas:
    meta_content.append(list(meta.attrs.values()))
print(meta_content)

# print(metas[0].attrs.values())

# TODO text classification https://medium.com/@bedigunjit/simple-guide-to-text-classification-nlp-using-svm-and-naive-bayes-with-python-421db3a72d34
for script in soup(["script", "style", "iframe"]):
    script.decompose()
print(' '.join(' '.join(list(soup.stripped_strings)).split())) # it is probably fine, but check if it extracts all textual content

text = ' '.join(' '.join(list(soup.stripped_strings)).split())

text = text.lower()

# may need to download some nltk modules
# import nltk
# nltk.download('wordnet')
# nltk.download('averaged_perceptron_tagger')
# nltk.download('stopwords')

from nltk import word_tokenize
text = word_tokenize(text)

# WordNetLemmatizer requires Pos tags to understand if the word is noun or verb or adjective etc. By default it is set to Noun
from collections import defaultdict
from nltk.corpus import wordnet as wn

tag_map = defaultdict(lambda : wn.NOUN)
tag_map['J'] = wn.ADJ
tag_map['V'] = wn.VERB
tag_map['R'] = wn.ADV

from nltk.stem import WordNetLemmatizer
from nltk import pos_tag

from nltk.corpus import stopwords


final_words = []
word_lemmatized = WordNetLemmatizer()

for word, tag in pos_tag(text):
    if word not in stopwords.words('english') and word.isalnum():
        word_final = word_lemmatized.lemmatize(word, tag_map[tag[0]])
        final_words.append(word_final)

print(final_words)

# TODO html parser to count tags
