import numpy as np
import pandas as pd
import tensorflow as tf
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.preprocessing import LabelEncoder
from train import extract_features, TrainModels

# Note: tried scaling the data and reducing the features with PCA, but the produced accuracies were too small
# TODO maybe use unittest python library instead of assertions?

data, labels = extract_features()
np.random.seed(42)
tf.random.set_seed(42)

# test the models by splitting the original dataset into test and train datasets
def test_models_split(data, labels):
    train_models = TrainModels(data, labels, True)
    models, test_features_and_labels = train_models.train_or_load()
    from sklearn.metrics import accuracy_score

    for model in models:
        pred = models[model].predict(test_features_and_labels[model][0])
        accuracy = 0
        if model == "neural_net":
            # since the y test labels for the neural network are also converted to a binary class matrix,
                                    # the process needs to be reversed
            accuracy = accuracy_score(pred, [np.argmax(y, axis=None, out=None) for y in test_features_and_labels[model][1]]) * 100
            print(models[model], " accuracy = ", accuracy)
        else:
            accuracy = accuracy_score(pred, test_features_and_labels[model][1]) * 100
            print(models[model], " accuracy = ", accuracy)

        # assert the predicted accuracies
        if model == "neural_net":
            assert np.isclose(accuracy, 78.76344086021506)
        if model == "svc":
            assert np.isclose(accuracy, 78.76344086021506)
        if model == "gnb":
            assert np.isclose(accuracy, 64.51612903225806)
        if model == "mnb":
            assert np.isclose(accuracy, 54.83870967741935)
        if model == "rand_forest":
            assert np.isclose(accuracy, 76.34408602150538)
        if model == "knn":
            assert np.isclose(accuracy, 35.752688172043015)

# test the real-world classification
import urllib.request
from bs4 import BeautifulSoup

def test_train_or_load(data, labels, webpage):
    train_models = TrainModels(data, labels)
    models = train_models.train_or_load()

    raw_webpage = webpage.read()
    decoded_webpage = raw_webpage.decode("utf8")
    webpage.close()

    from feature_extraction import extract_features_from_html

    features = extract_features_from_html(data, decoded_webpage, extract_features_from_html=False)

    label_encoder = LabelEncoder()
    label_encoder.fit(labels)

    # prediciton
    neural_net_pred = ""
    try:
        # This will work if the model was loaded from the files, because it is without the KerasClassifier wrapper
        # If the model was just trained inverse_transform will throw a ValueError
        neural_net_pred = label_encoder.inverse_transform(np.argmax(models["neural_net"].predict(features), axis=-1))
    except ValueError:
        neural_net_pred = label_encoder.inverse_transform(models["neural_net"].predict(features))
    print(neural_net_pred)
    assert neural_net_pred == "articles"

    svm_pred = label_encoder.inverse_transform(models["svm"].predict(features))
    print(svm_pred)
    assert svm_pred == "help"

    features = extract_features_from_html(data, decoded_webpage, extract_features_from_html=True)

    gnb_pred = label_encoder.inverse_transform(models["gnb"].predict(features))
    print(gnb_pred)
    assert gnb_pred == "help"

    mnb_pred = label_encoder.inverse_transform(models["mnb"].predict(features))
    print(mnb_pred)
    assert mnb_pred == "help"

    rand_forest_pred = label_encoder.inverse_transform(models["rand_forest"].predict(features))
    print(rand_forest_pred)
    assert rand_forest_pred == "articles"

    knn_pred = label_encoder.inverse_transform(models["knn"].predict(features))
    print(knn_pred)
    assert knn_pred == "help"


# tests
test_models_split(data, labels)
print("\n\n")

# if the data/models directory exists, delete it to test the training of the models
import shutil
import os
models_dir = "data/models"
if os.path.exists(models_dir) and os.path.isdir(models_dir):
    print(models_dir, "directory found. Deleting...", end="")
    shutil.rmtree(models_dir)
    print("Done")

# webpage = urllib.request.urlopen("https://www.python.org/downloads/")
webpage = urllib.request.urlopen("https://docs.python.org/3/library/html.parser.html")
# train, save, and test the models
test_train_or_load(data, labels, webpage)

# test the models saved above
webpage = urllib.request.urlopen("https://docs.python.org/3/library/html.parser.html")
test_train_or_load(data, labels, webpage)
