## Description
A search engine will benefit from a website classification model to differentiate the web pages into several categories. The rank of the web pages will be calculated based on its category.

## Dataset
The model is trained on the KI-04 dataset, as it was the only publicly available dataset with website genres that I could find, although it probably is outdated.

## Results
Trained only on the text of the webpages, the classifier achieved the following accuracies:
* Naive Bayes Accuracy = 72.58064516129032%
* SVM Accuracy = 77.41935483870968%

Trained on the text and meta tags of the webpages, the classifier achieved the following accuracies:
* Naive Bayes Accuracy = 75.26881720430107%
* SVM Accuracy = 79.3010752688172%

After I set the random seed to get consistent results, the classifier, trained on text only achieved:
* Naive Bayes Accuracy = 74.19354838709677%
* SVM Accuracy = 77.68817204301075%

Trained on both text and meta tag information (and the same random seed):
* Naive Bayes Accuracy = 73.65591397849462%
* SVM Accuracy = 78.76344086021506%

Trained a deep neural network using the text and meta tags information:
* Neural Network Accuracy = 76.34408602150538%

After setting the seed manually:
* Neural Network Accuracy = 77.95698924731182%

After inclusing the number of `<a>` tags as a feature, only the SVM improved a bit, while the neural network suffered greatly, so for now I decided to not include the `<a>` tag for the nn classifier.
* Naive Bayes Accuracy = 73.38709677419355%
* SVM Accuracy = 79.03225806451613%

After including all extracted information from the html tags, the svm training process became too slow, so the svm and the neural network were only trained with the textual and meta tag information.
**The SVM classifier may benefit from some of the extracted information, but further tests are necessary to pick the best (training time):accuracy ratio.**
I additionally included Gaussian Naive Bayes, Random Forest and K Nearest Neighbors classifiers. The achieved accuracies are:
* Gaussian Naive Bayes Accuracy = 64.51612903225806%
* Multinomial Naive Bayes Accuracy = 54.83870967741935%
* Random Forest Accuracy = 77.15053763440861%
* K Nearest Neighbors Accuracy = 35.752688172043015%