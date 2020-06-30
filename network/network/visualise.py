#!/usr/bin/env python
import numpy as np
import matplotlib.pyplot as plt
from scipy import stats
from network.decode import decode
import os

with open(os.path.join(os.path.dirname(__file__), '..', '..', 'generated_puzzles', '_examples.bin'), 'rb') as f:
    contents = f.read()

images, labels = decode(contents)

label_frequency = np.zeros(2)

for label in labels:
    label_frequency[int(label)] += 1

# fig, axs = plt.subplots(sharey=True, tight_layout=True)

# We can set the number of bins with the `bins` kwarg
# axs.hist(labels, bins=10)

plt.bar(range(0, 2), label_frequency, color='green')

plt.show()