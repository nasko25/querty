import numpy as np
import pandas as pd
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.preprocessing import LabelEncoder
from train import extract_features, TrainModels

# Note: tried scaling the data and reducing the features with PCA, but the produced accuracies were too small

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
train_models = TrainModels(data, labels)
models = train_models.train_or_load()
# temporary; very bad code to classify webpage from a given url; random forest performs decently
# TODO refactor
import urllib.request
from bs4 import BeautifulSoup

# fp = urllib.request.urlopen("https://www.python.org/downloads/")
fp = urllib.request.urlopen("https://docs.python.org/3/library/html.parser.html")
mybytes = fp.read()

mystr = mybytes.decode("utf8")
fp.close()
from feature_extraction import extract_text, extract_metas, extract_html_info

soup = BeautifulSoup(mystr, features="html5lib")

tf_idf_text = TfidfVectorizer(max_features=5000)
tf_idf_meta = TfidfVectorizer(max_features=5000)

tf_idf_text.fit(data["text"])
tf_idf_meta.fit(data["meta"])
t = tf_idf_text.transform([str(extract_text(soup))])
m = tf_idf_meta.transform([str(extract_metas(soup))])

t = pd.DataFrame(t.toarray())
m = pd.DataFrame(m.toarray())

x = pd.concat([t, m], axis = 1)

h = extract_html_info(mystr)
a = pd.DataFrame([h["a"]])
li = pd.DataFrame([h["li"]])
script = pd.DataFrame([h["script"]])
script_words = pd.DataFrame([h["script_words"]])
iframe = pd.DataFrame([h["iframe"]])
i = pd.DataFrame([h["input"]])

label_encoder = LabelEncoder()
label_encoder.fit(labels)
try:
    # This will work if the model was loaded from the files, because it is without the KerasClassifier wrapper
    # If the model was just trained inverse_transform will throw a ValueError
    print(label_encoder.inverse_transform(np.argmax(models["neural_net"].predict(x), axis=-1)))
except ValueError:
    print(label_encoder.inverse_transform(models["neural_net"].predict(x)))

print(label_encoder.inverse_transform(models["svm"].predict(x)))
x = pd.concat([x, a, li, script, script_words, iframe, i], axis = 1)

print(label_encoder.inverse_transform(models["gnb"].predict(x)))
print(label_encoder.inverse_transform(models["mnb"].predict(x)))
print(label_encoder.inverse_transform(models["rand_forest"].predict(x)))
print(label_encoder.inverse_transform(models["knn"].predict(x)))

# TODO delete /models and retrain to test the models without loading them from disk