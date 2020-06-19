#!/usr/bin/env python

from tensorflow import keras
import tensorflow as tf
import numpy as np
import os
from network.decode import decode
import datetime
from pathlib import Path

log_dir = "logs/fit/" + datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
tensorboard_callback = keras.callbacks.TensorBoard(log_dir=log_dir, histogram_freq=1, write_images=True)

with open(os.path.join(os.path.dirname(__file__), '..', '..', 'generated_puzzles', '_examples.bin'), 'rb') as f:
    contents = f.read()


images, labels = decode(contents)

model = keras.Sequential([
    keras.Input(shape=(8, 16, 3)),
    keras.layers.Conv2D(kernel_size=3, filters=16, strides=1, padding='same', activation='relu'),
    keras.layers.MaxPool2D(pool_size=(2, 2)),
    keras.layers.Conv2D(kernel_size=3, filters=32, strides=1, padding='same', activation='relu'),
    keras.layers.MaxPool2D(pool_size=(2, 2)),
    keras.layers.Conv2D(kernel_size=3, filters=64, strides=1, padding='same', activation='relu'),
    keras.layers.MaxPool2D(pool_size=(2, 2)),
    keras.layers.Flatten(),
    keras.layers.Dense(512, activation='relu'),
    keras.layers.Dense(129),
])

model.summary()

model.compile(optimizer=keras.optimizers.Adam(), loss=keras.losses.CategoricalCrossentropy(from_logits=True), metrics=[keras.metrics.CategoricalAccuracy()])

model.fit(images, labels, epochs=100, validation_split=0.2, callbacks=[
    tensorboard_callback,
    keras.callbacks.EarlyStopping(patience=10)
])

model.save(Path(__file__).parent.parent / 'model')
