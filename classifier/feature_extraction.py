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
import numpy as np
import pandas as pd
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.preprocessing import LabelEncoder
from sklearn.model_selection import train_test_split
from bs4 import BeautifulSoup

np.random.seed(42)

def extract_text(soup):
    # print(' '.join(soup.get_text().split()))   # still has some scripts

    # print("\n\n")

    # TODO text classification https://medium.com/@bedigunjit/simple-guide-to-text-classification-nlp-using-svm-and-naive-bayes-with-python-421db3a72d34
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

data_dir = "data/genre-corpus-04"
data = {
    "text": [],
    "meta" : []
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
    for subdir, dirs, files in os.walk(data_dir):
        dir = subdir.split("/")[-1]
        if dir in genres:
            for file_name in files:
                file = open(subdir + "/" + file_name, 'r')
                print(file.name)
                html = file.read()
                soup = BeautifulSoup(html, features="html5lib")
                data["text"].append(str(extract_text(soup)))
                data["meta"].append(str(extract_metas(soup)))
                labels.append(dir)
                file.close()
    f = open(data_saved_file, 'w')
    json.dump(data, f)
    f.close()

    f = open(labels_saved_file, 'w')
    json.dump(labels, f)
    f.close()

# random_file = "data/genre-corpus-04/articles/5440455052.html"

# f = open(random_file, 'r')
# html = f.read()
# f.close()

# TODO will need to manually extract the text?
# TODO strip space characters (and \t\n)
# h = html2text.HTML2Text()
# h.ignore_links = True
# h.ignore_images = True ?
# print(h.handle(html))

# TODO html parser to count tags

# split the text and meta tag information into test and train sets
x_train_text, x_test_text, x_train_meta, x_test_meta, y_train, y_test = train_test_split(data["text"], data["meta"], labels, test_size=0.3)

# encode labels into numerical values
label_encoder = LabelEncoder()
y_train = label_encoder.fit_transform(y_train)
y_test = label_encoder.fit_transform(y_test)

# vectorize the data's text and metadata fields using tf-idf
tf_idf_text = TfidfVectorizer(max_features=5000)
tf_idf_text.fit(data["text"])
tfidf_x_train_text = tf_idf_text.transform(x_train_text)
tfidf_x_test_text = tf_idf_text.transform(x_test_text)

tf_idf_meta = TfidfVectorizer(max_features=5000)
tf_idf_meta.fit(data["meta"])
tfidf_x_train_meta = tf_idf_meta.transform(x_train_meta)
tfidf_x_test_meta = tf_idf_meta.transform(x_test_meta)

# combine both train features into one training feature
df_train_text = pd.DataFrame(tfidf_x_train_text.toarray())
df_train_meta = pd.DataFrame(tfidf_x_train_meta.toarray())

train_features = pd.concat([df_train_text, df_train_meta], axis = 1)

# combine both test features into one testing feature
df_test_text = pd.DataFrame(tfidf_x_test_text.toarray())
df_test_meta = pd.DataFrame(tfidf_x_test_meta.toarray())

test_features = pd.concat([df_test_text, df_test_meta], axis = 1)

print(tf_idf_text.vocabulary_)
print(tfidf_x_train_text.shape)
print(np.array(x_train_text).shape)
print(np.array(y_train).shape)

# classifiers
# from sklearn import model_selection, naive_bayes, svm
# from sklearn.naive_bayes import MultinomialNB
# from sklearn.svm import SVC

# # naive bayes classifier
# nb = MultinomialNB()

# nb.fit(train_features, y_train)

# # prediction
# pred_nb = nb.predict(test_features)

# # print the accuracy
from sklearn.metrics import accuracy_score
# print("Naive Bayes Accuracy = ", accuracy_score(pred_nb, y_test) * 100, "%", sep = "")

# # svm classifier
# svm = SVC(C = 1.0, kernel = 'linear', degree = 3, gamma = 'auto')
# svm.fit(train_features, y_train)

# # prediction
# pred_svm = svm.predict(test_features)

# # print the accuracy
# print("SVM Accuracy = ",accuracy_score(pred_svm, y_test) * 100, "%", sep = "")

# deep neural network
from keras.models import Sequential
from keras.layers import Dense, Dropout
from keras.wrappers.scikit_learn import KerasClassifier
from keras.utils import np_utils

# Model Training
print ("Create model ... ")
def build_model():
    model = Sequential()
    model.add(Dense(256, input_dim = 10000, activation='relu'))
    model.add(Dropout(0.3))
    model.add(Dense(200, activation='relu'))
    model.add(Dropout(0.3))
    model.add(Dense(160, activation='relu'))
    model.add(Dropout(0.3))
    model.add(Dense(120, activation='relu'))
    model.add(Dropout(0.3))
    model.add(Dense(80, activation='relu'))
    model.add(Dropout(0.3))
    model.add(Dense(20, activation='relu'))
    model.add(Dropout(0.3))
    model.add(Dense(8, activation='softmax'))
    model.compile(loss='categorical_crossentropy', optimizer='adam', metrics=['accuracy'])
    model.summary()
    return model

print("Compile model ...")
estimator = KerasClassifier(build_fn=build_model, epochs=15, batch_size=128)
y_train_cat = np_utils.to_categorical(y_train, 8)
estimator.fit(train_features, y_train_cat)

# Predictions
print ("Predict on test data ... ")
pred_nn = estimator.predict(test_features)
y_pred = label_encoder.inverse_transform(pred_nn)

print("Neural Network Accuracy = ",accuracy_score(pred_nn, y_test) * 100, "%", sep = "")
# print(pred_nn, y_pred)