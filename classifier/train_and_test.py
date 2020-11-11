import numpy as np
import pandas as pd
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.preprocessing import LabelEncoder
from train import extract_features, TrainModels

# Note: tried scaling the data and reducing the features with PCA, but the produced accuracies were too small
# TODO refactor into functions for readability

data, labels = extract_features()

# test the models by splitting the original dataset into test and train datasets
train_models = TrainModels(data, labels, True)
models, test_features_and_labels = train_models.train_or_load()
from sklearn.metrics import accuracy_score

for model in models:
    pred = models[model].predict(test_features_and_labels[model][0])
    if model == "neural_net":
        # since the y test labels for the neural network are also converted to a binary class matrix,
                                                                                # the process needs to be reversed
        print(models[model], " accuracy = ", accuracy_score(pred, [np.argmax(y, axis=None, out=None) for y in test_features_and_labels[model][1]]) * 100)
    else:
        print(models[model], " accuracy = ", accuracy_score(pred, test_features_and_labels[model][1]) * 100)
print("\n\n")

# test the real-world classification
import urllib.request
from bs4 import BeautifulSoup

def test(webpage):
    train_models = TrainModels(data, labels)
    models = train_models.train_or_load()

    raw_webpage = webpage.read()

    decoded_webpage = raw_webpage.decode("utf8")

    webpage.close()
    from feature_extraction import extract_text, extract_metas, extract_html_info

    soup = BeautifulSoup(decoded_webpage, features="html5lib")

    # TODO extract this to a function; maybe in feature_extraction.py
    tf_idf_text = TfidfVectorizer(max_features=5000)
    tf_idf_meta = TfidfVectorizer(max_features=5000)

    tf_idf_text.fit(data["text"])
    tf_idf_meta.fit(data["meta"])
    text = tf_idf_text.transform([str(extract_text(soup))])
    meta = tf_idf_meta.transform([str(extract_metas(soup))])

    text = pd.DataFrame(text.toarray())
    meta = pd.DataFrame(meta.toarray())

    features = pd.concat([text, meta], axis = 1)

    html = extract_html_info(decoded_webpage)
    a = pd.DataFrame([html["a"]])
    li = pd.DataFrame([html["li"]])
    script = pd.DataFrame([html["script"]])
    script_words = pd.DataFrame([html["script_words"]])
    iframe = pd.DataFrame([html["iframe"]])
    i = pd.DataFrame([html["input"]])

    label_encoder = LabelEncoder()
    label_encoder.fit(labels)
    try:
        # This will work if the model was loaded from the files, because it is without the KerasClassifier wrapper
        # If the model was just trained inverse_transform will throw a ValueError
        print(label_encoder.inverse_transform(np.argmax(models["neural_net"].predict(features), axis=-1)))
    except ValueError:
        print(label_encoder.inverse_transform(models["neural_net"].predict(features)))

    print(label_encoder.inverse_transform(models["svm"].predict(features)))
    features = pd.concat([features, a, li, script, script_words, iframe, i], axis = 1)

    print(label_encoder.inverse_transform(models["gnb"].predict(features)))
    print(label_encoder.inverse_transform(models["mnb"].predict(features)))
    print(label_encoder.inverse_transform(models["rand_forest"].predict(features)))
    print(label_encoder.inverse_transform(models["knn"].predict(features)))

# if the data/models directory exists, delete it to test the training of the models
import shutil
import os
models_dir = "data/models"
if os.path.exists(models_dir) and os.path.isdir(models_dir):
    print(models_dir, "directory found. Deleting...", end="")
    # shutil.rmtree(models_dir)
    print("Done")

# webpage = urllib.request.urlopen("https://www.python.org/downloads/")
webpage = urllib.request.urlopen("https://docs.python.org/3/library/html.parser.html")
# train, save, and test the models
test(webpage)

# test the models saved above
webpage = urllib.request.urlopen("https://docs.python.org/3/library/html.parser.html")
test(webpage)
# TODO maybe assert that the returned values from the test function are equal to some expected values
# for example, the accuracies for each model are known, and predicted website genres for the given webpage are known