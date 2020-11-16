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
    # model.add(Dense(20, activation='relu'))
    # model.add(Dropout(0.3))
    model.add(Dense(8, activation='softmax'))
    model.compile(loss='categorical_crossentropy', optimizer='adam', metrics=['accuracy'])
    # model.summary()
    return model

# extract the features
def extract_features():
    # extract the features from the dataset
    data, labels = feature_extraction.extract_features()

    return data, labels
class TrainModels():
    def __init__(self, X, y, test=False):
        self._trained_models_path = "data/models"
        self.data = X
        self.labels = y

        # used to determin if we need to split the dataset into training and testing datasets
        self.test = test

        # used to encode labels into numerical values
        self.label_encoder = LabelEncoder()
        self.label_encoder.fit(y)

        # will need to vectorize the data's text and metadata fields using tf-idf
        self.tf_idf_text = TfidfVectorizer(max_features=5000)
        self.tf_idf_meta = TfidfVectorizer(max_features=5000)

        self.tf_idf_text.fit(self.data["text"])
        self.tf_idf_meta.fit(self.data["meta"])

    # Preprocess the datasets
    def _preprocess(self, data, labels, model_name):
        labels_encoded = self.label_encoder.transform(labels)

        # vectorize the data's text and metadata fields using tf-idf
        tfidf_x_text = self.tf_idf_text.transform(data["text"])
        tfidf_x_meta = self.tf_idf_meta.transform(data["meta"])

        # combine text and meta features into one
        df_text = pd.DataFrame(tfidf_x_text.toarray())
        df_meta = pd.DataFrame(tfidf_x_meta.toarray())
        features = pd.concat([df_text, df_meta], axis = 1)

        # the neural network and svm were trained with only the text and meta tag information extracted from the web pages

        if model_name == "neural_net":
            # the labels for the neural network will need to be converted to a binary class matrix
            labels_cat = np_utils.to_categorical(labels_encoded, 8)
            return features, labels_cat
        # the svm classifier was too slow with the additional html extracted features, so it is only trained with the text and meta tag information 
        elif model_name == "svm":
            return features, labels_encoded

        else:
            # all other models use the html extracted information as well

            # since data["html"] is a list of dictionaries, a helper function to extract all dictionary values from the list is needed
            # use -1 as a default value if the key is not present inside the dictionary (should never be the case in reality)
            get_values = lambda key, values: [dictionary[key] if key in dictionary else -1 for dictionary in values]

            # convert the features extracted from the html code to pandas dataframes
            # (they are used to train some of the models)
            x_html = data["html"]
            feat_a = pd.DataFrame(get_values("a", x_html))
            feat_li = pd.DataFrame(get_values("li", x_html))
            feat_script = pd.DataFrame(get_values("script", x_html))
            feat_script_words = pd.DataFrame(get_values("script_words", x_html))
            feat_iframe = pd.DataFrame(get_values("iframe", x_html))
            feat_input = pd.DataFrame(get_values("input", x_html))

            # include the additional information from the html tags
            features = pd.concat([features, feat_a, feat_li, feat_script, feat_script_words, feat_iframe, feat_input], axis = 1)

            return features, labels_encoded

    # preprocess the datasets and split into train and test sets
    def _preprocess_split(self, data, labels, model_name):
        # split the text, meta tag information, and the html extracted information into test and train sets
        x_train_text, x_test_text, x_train_meta, x_test_meta, x_train_html, x_test_html, y_train, y_test = train_test_split(data["text"], data["meta"], data["html"], labels, test_size=0.3, random_state=42)

        train_data = {
            "text": x_train_text,
            "meta": x_train_meta,
            "html": x_train_html,
        }
        test_data = {
            "text": x_test_text,
            "meta": x_test_meta,
            "html": x_test_html,
        }

        train_features, train_labels = self._preprocess(train_data, y_train, model_name)
        test_features, test_labels = self._preprocess(test_data, y_test, model_name)

        return train_features, test_features, train_labels, test_labels

    # if the model was not saved to the specified folder, train it
    def _train_model(self, model, model_name):
        data, labels = self.data, self.labels

        # if test is set to True, split the data into training and testing datasets, train the model, and return the test sets to be tested later
        if self.test:
            train_features, test_features, train_labels, test_labels = self._preprocess_split(data, labels, model_name)
            model.fit(train_features, train_labels)
            return test_features, test_labels

        # otherwise just preprocess the dataset and fit the model
        else:
            train_features, train_labels = self._preprocess(data, labels, model_name)
            model.fit(train_features, train_labels)

        # # TODO remove:
        # # predict and print accuracies
        # from sklearn.metrics import accuracy_score

        # pred = model.predict(test_features)
        # print(model, " accuracy = ", accuracy_score(pred, y_test) * 100)

    def _save_model(self, model, filename, nn=False):
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

    def _load_model(self, filename, nn=False):
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

    def train_or_load(self):
        models = {
            "neural_net": KerasClassifier(build_fn=_build_model, epochs=25, batch_size=64),
            "svm": SVC(C = 1.0, kernel = 'linear', degree = 3, gamma = 'auto'),
            "gnb": GaussianNB(),
            "mnb": MultinomialNB(alpha=0.000001),
            "rand_forest": RandomForestClassifier(),
            "knn": KNeighborsClassifier(n_neighbors=50)
        }

        # if the test flag is set, will need to save the test features and labels in a dictionary of tuples
        # so initilize that dictionary
        if self.test:
            test_features_and_labels = {}

        for model in models:
            filename = self._trained_models_path + "/" + model
                # if this is a test, train the models, because the saved models were trained on a different training set
            if not self.test and \
            (
                        # for the classifiers that are not a neural network
                    (os.path.exists(filename + ".sav") and os.path.isfile(filename + ".sav"))
                        # the neural networks is saved in two files
                    or (os.path.exists(filename + ".json") and os.path.isfile(filename + ".json") and os.path.exists(filename + ".h5") and os.path.isfile(filename + ".h5"))
            ):

                print("File", filename, "found.")
                print("Model", model, "is already trained.", "Loading it in the models dictionary.\n")
                if model == "neural_net":
                    models[model] = self._load_model(filename, nn=True)
                else:
                    models[model] = self._load_model(filename)
            else:
                # TODO remove in production
                # (moved it to train_and_test.py, so it will still be used when testing)
                # np.random.seed(42)
                # tf.random.set_seed(42)
                # -------------------------

                print("Model", model, "was not previously trained.")
                print("Training...")
                if self.test:
                    test_features, test_labels = self._train_model(models[model], model)
                    test_features_and_labels[model] = (test_features, test_labels)
                    # no need to save test models
                else:
                    self._train_model(models[model], model)
                    print("Training complete. Saving the model...")
                    if model == "neural_net":
                        self._save_model(models[model].model, filename, nn=True)
                    else:
                        self._save_model(models[model], filename)
                    print("Model saved.\n")

        # if test is True, also return the test_features and test_labels for each model
        if self.test:
            return models, test_features_and_labels
        else:
            return models
