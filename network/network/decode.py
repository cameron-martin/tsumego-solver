import numpy as np

BYTES_PER_RECORD = 49

def decode(stream: bytes):
    num_bytes = len(stream)
    if num_bytes % BYTES_PER_RECORD != 0:
        raise Exception(f'The file does not contain a multiple of {BYTES_PER_RECORD} bytes')
    num_records = num_bytes // BYTES_PER_RECORD
    images = np.empty((num_records, 8, 16, 3), dtype=np.float32)
    labels = np.zeros((num_records, 1, 1, 129))
    bytes_offset = 0
    for i in range(0, num_records):
        current_byte = stream[bytes_offset]
        mask = 0x80
        for channel in range(0, 3):
            for y in range(0, 8):
                for x in range(0, 8):
                    images[i][y][x][channel] = 1 if mask & current_byte else 0
                    mask = mask >> 1
                bytes_offset += 1
                current_byte = stream[bytes_offset]
                mask = 0x80
                for x in range(8, 16):
                    images[i][y][x][channel] = 1 if mask & current_byte else 0
                    mask = mask >> 1
                bytes_offset += 1
                current_byte = stream[bytes_offset]
                mask = 0x80
        labels[i][0][0][current_byte] = 1
        bytes_offset += 1
    return images, labels
