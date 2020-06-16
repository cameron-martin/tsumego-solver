#!/usr/bin/env python

from tensorflow import keras
import numpy as np
import os
from network.decode import decode

with open(os.path.join(os.path.dirname(__file__), '..', '..', 'generated_puzzles', '_examples.bin'), 'rb') as f:
    contents = f.read()

images, labels = decode(contents)
inputs = keras.Input(shape=(8, 16, 3))
x = keras.layers.Conv2D(kernel_size=3, filters=8, strides=1, padding='same', activation='relu')(inputs)
x = keras.layers.MaxPool2D(pool_size=(1, 2))(x)
x = keras.layers.Conv2D(kernel_size=3, filters=16, strides=1, padding='same', activation='relu')(x)
x = keras.layers.MaxPool2D(pool_size=(2, 2))(x)
x = keras.layers.Conv2D(kernel_size=3, filters=32, strides=1, padding='same', activation='relu')(x)
x = keras.layers.MaxPool2D(pool_size=(2, 2))(x)
x = keras.layers.Conv2D(kernel_size=3, filters=64, strides=1, padding='same', activation='relu')(x)
x = keras.layers.MaxPool2D(pool_size=(2, 2))(x)
x = keras.layers.Conv2D(kernel_size=3, filters=129, strides=1, padding='same', activation='relu')(x)

model = keras.Model(inputs=inputs, outputs=x)

model.summary()

model.compile(optimizer=keras.optimizers.Adam(), loss=keras.losses.CategoricalCrossentropy())

model.fit(images, labels, epochs=50, validation_split=0.2)
