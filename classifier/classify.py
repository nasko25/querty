import numpy as np
from urllib.request import Request, urlopen
from urllib.error import URLError, HTTPError
from train import TrainModels

def _classify(webpage):
    # decode the webpage and pre-process the features
    raw_webpage = webpage.read()
    decoded_webpage = raw_webpage.decode("utf8")
    webpage.close()

    # extract features and train (or load) the models
    train_models, features, features_with_html_info = TrainModels.auto_extract_features(decoded_webpage)
    models = train_models.train_or_load()

    # predicitons
    # neural_net_pred = ""
    # try:
        # This will work if the model was loaded from the files, because it is without the KerasClassifier wrapper
        # If the model was just trained inverse_transform will throw a ValueError
    #     neural_net_pred = train_models.label_encoder.inverse_transform(np.argmax(models["neural_net"].predict(features), axis=-1))
    # except ValueError:
    #     neural_net_pred = train_models.label_encoder.inverse_transform(models["neural_net"].predict(features))

    # svm_pred = train_models.label_encoder.inverse_transform(models["svm"].predict(features))

    # gnb_pred = train_models.label_encoder.inverse_transform(models["gnb"].predict(features_with_html_info))
    # mnb_pred = train_models.label_encoder.inverse_transform(models["mnb"].predict(features_with_html_info))
    rand_forest_pred = train_models.label_encoder.inverse_transform(models["rand_forest"].predict(features_with_html_info))
    # knn_pred = train_models.label_encoder.inverse_transform(models["knn"].predict(features_with_html_info))

    # TODO return accuracy based on some formula on what algorithm predicted what?
    # (but still use only rand_forest, unless probably all other predicitions are the same and only rand_forest is different?)
    return rand_forest_pred

'''
    classify a webpage from its url
        - check the validity of the user input and call internal classification method
        - returns None if the url is not valid, could not be opened, or returned a status code != 200
    requires internal function _classify

    parameters:
        - url of the website to classify
'''
def classify(url):
    webpage_classified = None 
    try:
        req = Request(url, headers = {'User-Agent':'Mozilla/5.0'})
        webpage = urlopen(req)
    except HTTPError as e:
        print('Error code: ', e.code)
        return None
    except URLError as e:
        print('Reason: ', e.reason)
        return None
    except ValueError:
        print("Invalid url.")
    else:
        print('good!')
        webpage_classified = _classify(webpage)

    return webpage_classified.tolist()

# classification = classify("https://www.rust-lang.org")
# if classification != None:
#     print("Type of website:", classification)

# classification = classify("https://python.org")
# if classification != None:
#     print("Type of website:", classification)
