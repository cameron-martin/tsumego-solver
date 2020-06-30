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

with open(os.path.join(os.path.dirname(__file__), '..', '..', 'generated_puzzles', '_examples_val.bin'), 'rb') as f:
    validation_contents = f.read()

images, labels = decode(contents)
validation_data = decode(validation_contents)

model = keras.Sequential([
    keras.Input(shape=(8, 16, 3)),
    keras.layers.Conv2D(kernel_size=3, filters=16, strides=1, padding='same', activation='relu'),
    # keras.layers.BatchNormalization(),
    keras.layers.MaxPool2D(pool_size=(2, 2)),
    keras.layers.Conv2D(kernel_size=3, filters=32, strides=1, padding='same', activation='relu'),
    keras.layers.MaxPool2D(pool_size=(2, 2)),
    keras.layers.Conv2D(kernel_size=3, filters=64, strides=1, padding='same', activation='relu'),
    keras.layers.MaxPool2D(pool_size=(2, 2)),
    keras.layers.Flatten(),
    # keras.layers.Dropout(0.2),
    keras.layers.Dense(64, activation='relu'),
    # keras.layers.Dropout(0.2),
    keras.layers.Dense(1, activation='sigmoid'),
])

model.summary()

model.compile(optimizer=keras.optimizers.Adam(), loss=keras.losses.BinaryCrossentropy(), metrics=[
    keras.metrics.BinaryAccuracy()])

model.fit(images, labels, epochs=100, validation_data=validation_data, callbacks=[
    tensorboard_callback,
    keras.callbacks.EarlyStopping(patience=2, restore_best_weights=True)
])

model.save(Path(__file__).parent.parent / 'model')
