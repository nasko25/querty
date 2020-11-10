import pickle
import os
import feature_extraction
import pandas as pd
import numpy as np
import tensorflow as tf

from sklearn.model_selection import train_test_split
from sklearn.preprocessing import LabelEncoder
from sklearn.feature_extraction.text import TfidfVectorizer

from keras.wrappers.scikit_learn import KerasClassifier
from keras.utils import np_utils
from keras.models import Sequential, model_from_json
from keras.layers import Dense, Dropout
from sklearn.svm import SVC
from sklearn.naive_bayes import GaussianNB
from sklearn.naive_bayes import MultinomialNB
from sklearn.ensemble import RandomForestClassifier
from sklearn.neighbors import KNeighborsClassifier

_trained_models_path = "data/models"

# the model used by the neural network
def _build_model():
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
    # model.summary()
    return model

# if the model was not saved to the specified folder, train it
def _train_model(model, model_name):
    # extract the features from the dataset
    data, labels = feature_extraction.extract_features()

    # TODO don't split in production
    # split the text, meta tag information, and the html extracted information into test and train sets
    x_train_text, x_test_text, x_train_meta, x_test_meta, x_train_html, x_test_html, y_train, y_test = train_test_split(data["text"], data["meta"], data["html"], labels, test_size=0.3)

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

    # combine text and meta train features into one training feature
    df_train_text = pd.DataFrame(tfidf_x_train_text.toarray())
    df_train_meta = pd.DataFrame(tfidf_x_train_meta.toarray())
    train_features = pd.concat([df_train_text, df_train_meta], axis = 1)

    # combine text and meta test features into one testing feature
    df_test_text = pd.DataFrame(tfidf_x_test_text.toarray())
    df_test_meta = pd.DataFrame(tfidf_x_test_meta.toarray())
    test_features = pd.concat([df_test_text, df_test_meta], axis = 1)

    # the neural network and svm were trained with only the text and meta tag information extracted from the web pages
    if model_name == "neural_net":
        # the labels for the neural network will need to be converted to a binary class matrix
        y_train_cat = np_utils.to_categorical(y_train, 8)
        model.fit(train_features, y_train_cat)

    # the svm classifier was too slow with the additional html extracted features, so it is only trained with the text and meta tag information 
    elif model_name == "svm":
        model.fit(train_features, y_train)
    else:
        # all other models use the html extracted information as well

        # since data["html"] is a list of dictionaries, a helper function to extract all dictionary values from the list is needed
        # use -1 as a default value if the key is not present inside the dictionary (should never be the case in reality)
        get_values = lambda key, values: [dictionary[key] if key in dictionary else -1 for dictionary in values]

        # convert the features extracted from the html code to pandas dataframes
        # (they are used to train some of the models)
        train_a = pd.DataFrame(get_values("a", x_train_html))
        train_li = pd.DataFrame(get_values("li", x_train_html))
        train_script = pd.DataFrame(get_values("script", x_train_html))
        train_script_words = pd.DataFrame(get_values("script_words", x_train_html))
        train_iframe = pd.DataFrame(get_values("iframe", x_train_html))
        train_input = pd.DataFrame(get_values("input", x_train_html))
        
        test_a = pd.DataFrame(get_values("a", x_test_html))
        test_li = pd.DataFrame(get_values("li", x_test_html))
        test_script = pd.DataFrame(get_values("script", x_test_html))
        test_script_words = pd.DataFrame(get_values("script_words", x_test_html))
        test_iframe = pd.DataFrame(get_values("iframe", x_test_html))
        test_input = pd.DataFrame(get_values("input", x_test_html))

        # include the additional information from the html tags
        train_features = pd.concat([train_features, train_a, train_li, train_script, train_script_words, train_iframe, train_input], axis = 1)
        test_features = pd.concat([test_features, test_a, test_li, test_script, test_script_words, test_iframe, test_input], axis = 1)

        model.fit(train_features, y_train)

    # TODO remove:
    # predict and print accuracies
    from sklearn.metrics import accuracy_score

    pred = model.predict(test_features)
    print(model, " accuracy = ", accuracy_score(pred, y_test) * 100)

def _save_model(model, filename, nn=False):
    # if the directories in the path of `filename` don't exist, create them
    if not os.path.exists(os.path.dirname(filename)):
        print("Directory", os.path.dirname(filename), "does not exist. Creating it now... ", end="")
        os.makedirs(os.path.dirname(filename), exist_ok=True)
        print("Done")

    # if it is a neural network model, don't use pickle, but export is as json
    if nn:
        f = open(filename + ".json", "w")
        # save the model
        f.write(model.to_json())
        # save the weights as well
        model.save_weights(filename + ".h5", overwrite=True)
    else:
        f = open(filename + ".sav", "wb")
        pickle.dump(model, f)

    f.close()

def _load_model(filename, nn=False):
    if nn:
        f = open(filename + ".json", "r")
        model = model_from_json(f.read())
        model.load_weights(filename + ".h5")
        model.compile(loss='categorical_crossentropy', optimizer='adam', metrics=['accuracy'])
    else:
        f = open(filename + ".sav", "rb")
        model = pickle.load(f)

    f.close()
    return model

def train_or_load():
    models = {
        "neural_net": KerasClassifier(build_fn=_build_model, epochs=25, batch_size=64),
        "svm": SVC(C = 1.0, kernel = 'linear', degree = 3, gamma = 'auto'),
        "gnb": GaussianNB(),
        "mnb": MultinomialNB(alpha=0.000001),
        "rand_forest": RandomForestClassifier(),
        "knn": KNeighborsClassifier(n_neighbors=50)
    }

    for model in models:
        filename = _trained_models_path + "/" + model
            # for the classifiers that are not a neural network
        if (os.path.exists(filename + ".sav") and os.path.isfile(filename + ".sav")) or (os.path.exists(filename + ".json") and os.path.isfile(filename + ".json") and os.path.exists(filename + ".h5") and os.path.isfile(filename + ".h5")):
            print("File", filename, "found.")
            print("Model", model, "is already trained.", "Loading it in the models dictionary.\n")
            if model == "neural_net":
                models[model] = _load_model(filename, nn=True)
            else:
                models[model] = _load_model(filename)
        else:
            # TODO remove in production
            np.random.seed(42)
            tf.random.set_seed(42)

            print("Model", model, "was not previously trained.")
            print("Training...")
            _train_model(models[model], model)
            print("Training complete. Saving the model...")
            if model == "neural_net":
                _save_model(models[model].model, filename, nn=True)
            else:
                _save_model(models[model], filename)
            print("Model saved.\n")

    return models

models = train_or_load()
# temporary; very bad code to classify webpage from a given url for random forest
# TODO remove
import urllib.request
from bs4 import BeautifulSoup

fp = urllib.request.urlopen("https://www.python.org/downloads/")
fp = urllib.request.urlopen("https://docs.python.org/3/library/html.parser.html")
mybytes = fp.read()

mystr = mybytes.decode("utf8")
fp.close()
from feature_extraction import extract_text, extract_metas, extract_html_info

soup = BeautifulSoup(mystr, features="html5lib")

tf_idf_text = TfidfVectorizer(max_features=5000)
tf_idf_meta = TfidfVectorizer(max_features=5000)
data, labels = feature_extraction.extract_features()
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

x = pd.concat([x, a, li, script, script_words, iframe, i], axis = 1)
print(label_encoder.inverse_transform(models["rand_forest"].predict(x)))