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

After I set the random seed to get consistent results, the classifier, trained on text and meta tags achieved:
* Naive Bayes Accuracy = 73.65591397849462%
* SVM Accuracy = 78.76344086021506%