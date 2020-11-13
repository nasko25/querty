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
import json
from html_extract_info import HTMLInfoExtractor
from bs4 import BeautifulSoup

def extract_text(soup):
    # print(' '.join(soup.get_text().split()))   # still has some scripts

    # print("\n\n")

    for script in soup(["script", "style", "iframe"]):
        script.decompose()
    # print(' '.join(' '.join(list(soup.stripped_strings)).split())) # it is probably fine, but check if it extracts all textual content

    print("\t* extracting text")
    text = ' '.join(' '.join(list(soup.stripped_strings)).split())
    print("\t\t- [done]")

    print("\t* processing")
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

    print("\t\t- [done]")
    # print(final_words)
    return final_words

def extract_metas(soup):
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
    return meta_content

    # print(metas[0].attrs.values())


def extract_html_info(html):
    parser = HTMLInfoExtractor()
    parser.feed(html)
    return parser.extract()

# TODO make the functions above private ^ (_)

def extract_features():
    data_dir = "data/genre-corpus-04"
    data = {
        "text": [],
        "meta" : [],
        "html": []
    }
    labels = []

    # save the data and labels variables in a json file to access it without performing the text preprocessing again and again
    data_saved_file = "data/data.json"
    labels_saved_file = "data/data_labels.json"

    if (os.path.exists(data_saved_file) and os.path.isfile(data_saved_file)) and (os.path.exists(labels_saved_file) and os.path.isfile(labels_saved_file)):
        f = open(data_saved_file, 'r')
        data = json.load(f)
        f.close()

        f = open(labels_saved_file, 'r')
        labels = json.load(f)
        f.close()
    else:
        # save all html documents in a dictionary of lists
        for subdir, _, files in os.walk(data_dir):
            dir = subdir.split("/")[-1]
            if dir in genres:
                for file_name in files:
                    file = open(subdir + "/" + file_name, 'r')
                    print(file.name)
                    html = file.read()
                    soup = BeautifulSoup(html, features="html5lib")
                    data["text"].append(str(extract_text(soup)))
                    data["meta"].append(str(extract_metas(soup)))
                    data["html"].append(extract_html_info(html))
                    labels.append(dir)
                    file.close()
        f = open(data_saved_file, 'w')
        json.dump(data, f)
        f.close()

        f = open(labels_saved_file, 'w')
        json.dump(labels, f)
        f.close()

    return data, labels

import pandas as pd
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.preprocessing import LabelEncoder

def extract_features_from_html(data, webpage, extract_features_from_html=True):
    raw_webpage = webpage.read()
    decoded_webpage = raw_webpage.decode("utf8")
    webpage.close()

    soup = BeautifulSoup(decoded_webpage, features="html5lib")

    tf_idf_text = TfidfVectorizer(max_features=5000)
    tf_idf_meta = TfidfVectorizer(max_features=5000)

    tf_idf_text.fit(data["text"])
    tf_idf_meta.fit(data["meta"])
    text = tf_idf_text.transform([str(extract_text(soup))])
    meta = tf_idf_meta.transform([str(extract_metas(soup))])

    text = pd.DataFrame(text.toarray())
    meta = pd.DataFrame(meta.toarray())

    features = pd.concat([text, meta], axis = 1)

    # extract the features from html
    if extract_features_from_html:
        html = extract_html_info(decoded_webpage)
        a = pd.DataFrame([html["a"]])
        li = pd.DataFrame([html["li"]])
        script = pd.DataFrame([html["script"]])
        script_words = pd.DataFrame([html["script_words"]])
        iframe = pd.DataFrame([html["iframe"]])
        i = pd.DataFrame([html["input"]])

        features = pd.concat([features, a, li, script, script_words, iframe, i], axis = 1)

    return features
