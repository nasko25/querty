import feature_extraction
import numpy as np
import pandas as pd
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.preprocessing import LabelEncoder
from sklearn.model_selection import train_test_split

np.random.seed(42)

# since data["html"] is a list of dictionaries, a helper function to extract all dictionary values from the list is needed
# use -1 as a default value if the key is not present inside the dictionary (should never be the case in reality)
get_values = lambda key, values: [dictionary[key] if key in dictionary else -1 for dictionary in values]

data, labels = feature_extraction.extract_features()

# split the text and meta tag information into test and train sets
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

# combine both train features into one training feature
df_train_text = pd.DataFrame(tfidf_x_train_text.toarray())
df_train_meta = pd.DataFrame(tfidf_x_train_meta.toarray())
train_a = pd.DataFrame(get_values("a", x_train_html))

train_features = pd.concat([df_train_text, df_train_meta], axis = 1)

# combine both test features into one testing feature
df_test_text = pd.DataFrame(tfidf_x_test_text.toarray())
df_test_meta = pd.DataFrame(tfidf_x_test_meta.toarray())
test_a = pd.DataFrame(get_values("a", x_test_html))

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

# include the a tag
train_features = pd.concat([train_features, train_a], axis = 1)
test_features = pd.concat([test_features, test_a], axis = 1)


# text classification inspired by https://medium.com/@bedigunjit/simple-guide-to-text-classification-nlp-using-svm-and-naive-bayes-with-python-421db3a72d34
# classifiers
from sklearn import model_selection, naive_bayes, svm
from sklearn.naive_bayes import MultinomialNB
from sklearn.svm import SVC

# naive bayes classifier
nb = MultinomialNB(alpha=0.001)

nb.fit(train_features, y_train)

# prediction
pred_nb = nb.predict(test_features)

# print the accuracy
print("Naive Bayes Accuracy = ", accuracy_score(pred_nb, y_test) * 100, "%", sep = "")

# svm classifier
svm = SVC(C = 1.0, kernel = 'linear', degree = 3, gamma = 'auto')
svm.fit(train_features, y_train)

# prediction
pred_svm = svm.predict(test_features)

# print the accuracy
print("SVM Accuracy = ",accuracy_score(pred_svm, y_test) * 100, "%", sep = "")
