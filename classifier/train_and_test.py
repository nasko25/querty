import feature_extraction
import numpy as np
import pandas as pd
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.preprocessing import LabelEncoder
from sklearn.model_selection import train_test_split

# TODO refactor and use train.py for training

np.random.seed(42)

# since data["html"] is a list of dictionaries, a helper function to extract all dictionary values from the list is needed
# use -1 as a default value if the key is not present inside the dictionary (should never be the case in reality)
get_values = lambda key, values: [dictionary[key] if key in dictionary else -1 for dictionary in values]

data, labels = feature_extraction.extract_features()

# split the text and meta tag information into test and train sets
x_train_text, x_test_text, x_train_meta, x_test_meta, x_train_html, x_test_html, y_train, y_test = train_test_split(data["text"], data["meta"], data["html"], labels, test_size=0.3, random_state=42)

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
train_a = pd.DataFrame(get_values("a", x_train_html))
train_li = pd.DataFrame(get_values("li", x_train_html))
train_script = pd.DataFrame(get_values("script", x_train_html))
train_script_words = pd.DataFrame(get_values("script_words", x_train_html))
train_iframe = pd.DataFrame(get_values("iframe", x_train_html))
train_input = pd.DataFrame(get_values("input", x_train_html))

train_features = pd.concat([df_train_text, df_train_meta], axis = 1)

# combine both test features into one testing feature
df_test_text = pd.DataFrame(tfidf_x_test_text.toarray())
df_test_meta = pd.DataFrame(tfidf_x_test_meta.toarray())
test_a = pd.DataFrame(get_values("a", x_test_html))
test_li = pd.DataFrame(get_values("li", x_test_html))
test_script = pd.DataFrame(get_values("script", x_test_html))
test_script_words = pd.DataFrame(get_values("script_words", x_test_html))
test_iframe = pd.DataFrame(get_values("iframe", x_test_html))
test_input = pd.DataFrame(get_values("input", x_test_html))

test_features = pd.concat([df_test_text, df_test_meta], axis = 1)

print(tf_idf_text.vocabulary_)
print(tfidf_x_train_text.shape)
print(np.array(x_train_text).shape)
print(np.array(y_train).shape)


# deep neural network classifier
import tensorflow as tf
from keras.models import Sequential
from keras.layers import Dense, Dropout
from keras.wrappers.scikit_learn import KerasClassifier
from keras.utils import np_utils
from sklearn.metrics import accuracy_score

# tried scaling the data and reducing the features with PCA, but the produced accuracies were too small
# from sklearn.preprocessing import StandardScaler
# from sklearn.decomposition import PCA

# scaler = StandardScaler()

# scaler.fit(train_features)
# train_features = scaler.transform(train_features)
# test_features = scaler.transform(test_features)


# pca = PCA(n_components=100)

# pca.fit(train_features)
# train_features = pca.transform(train_features)
# test_features = pca.transform(test_features)

# the neural network and svm were trained with only the text and meta tag information extracted from the web pages
tf.random.set_seed(42)

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
    # model.summary()
    return model

estimator = KerasClassifier(build_fn=build_model, epochs=25, batch_size=64)
y_train_cat = np_utils.to_categorical(y_train, 8)
estimator.fit(train_features, y_train_cat)

# prediction
pred_nn = estimator.predict(test_features)
y_pred = label_encoder.inverse_transform(pred_nn)

# print the accuracy
print("Neural Network Accuracy = ", accuracy_score(pred_nn, y_test) * 100, "%", sep = "")
# print(pred_nn, y_pred)


from sklearn.svm import SVC

# the svm classifier was too slow with the additional html extracted features, so it is only trained with the text and meta tag information 
# svm classifier
svm = SVC(C = 1.0, kernel = 'linear', degree = 3, gamma = 'auto')
svm.fit(train_features, y_train)

# prediction
pred_svm = svm.predict(test_features)

# print the accuracy
print("SVM Accuracy = ",accuracy_score(pred_svm, y_test) * 100, "%", sep = "")


# include the additional information from the html tags
train_features = pd.concat([train_features, train_a, train_li, train_script, train_script_words, train_iframe, train_input], axis = 1)
test_features = pd.concat([test_features, test_a, test_li, test_script, test_script_words, test_iframe, test_input], axis = 1)

# text classification inspired by https://medium.com/@bedigunjit/simple-guide-to-text-classification-nlp-using-svm-and-naive-bayes-with-python-421db3a72d34
# classifiers
from sklearn import model_selection, naive_bayes
from sklearn.naive_bayes import GaussianNB

# Gaussian naive bayes classifier
gnb = GaussianNB()

gnb.fit(train_features, y_train)

# prediction
pred_gnb = gnb.predict(test_features)

# print the accuracy
print("Gaussian Naive Bayes Accuracy = ", accuracy_score(pred_gnb, y_test) * 100, "%", sep = "")

from sklearn.naive_bayes import MultinomialNB

# multinomial naive bayes classifier
mnb = MultinomialNB(alpha=0.000001)

mnb.fit(train_features, y_train)

# prediction
pred_mnb = mnb.predict(test_features)

# print the accuracy
print("Multinomial Naive Bayes Accuracy = ", accuracy_score(pred_mnb, y_test) * 100, "%", sep = "")

from sklearn.ensemble import RandomForestClassifier

# random forest classifier
rand_forest = RandomForestClassifier()

rand_forest.fit(train_features, y_train)

# prediction
pred_rand_forest = rand_forest.predict(test_features)

# print the accuracy
print("Random Forest Accuracy = ", accuracy_score(pred_rand_forest, y_test) * 100, "%", sep = "")


from sklearn.neighbors import KNeighborsClassifier

# knn classifier
knn = KNeighborsClassifier(n_neighbors=50)

knn.fit(train_features, y_train)

# prediction
pred_knn = knn.predict(test_features)

# print the accuracy
print("K Nearest Neighbors Accuracy = ", accuracy_score(pred_knn, y_test) * 100, "%", sep = "")

# TODO save the fitted models to avoid training them over and over again
# TODO classify.py and train.py should be separate; this file could be called test_models.py

# temporary; bad code to classify webpage from a given url. Random forest performed decently.
import urllib.request
from bs4 import BeautifulSoup

fp = urllib.request.urlopen("https://www.python.org/downloads/")
fp = urllib.request.urlopen("https://docs.python.org/3/library/html.parser.html")
mybytes = fp.read()

mystr = mybytes.decode("utf8")
fp.close()
from feature_extraction import extract_text, extract_metas, extract_html_info

soup = BeautifulSoup(mystr, features="html5lib")
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

print(label_encoder.inverse_transform(estimator.predict(x)))
print(label_encoder.inverse_transform(svm.predict(x)))
print()
x = pd.concat([x, a, li, script, script_words, iframe, i], axis = 1)
print(label_encoder.inverse_transform(gnb.predict(x)))
print(label_encoder.inverse_transform(mnb.predict(x)))
print(label_encoder.inverse_transform(rand_forest.predict(x)))
print(label_encoder.inverse_transform(knn.predict(x)))
