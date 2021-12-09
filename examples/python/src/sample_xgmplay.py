# -*- coding: utf-8 -*-
# license:BSD-3-Clause
#
# XGM Play Example
#
import pygame
from wasm.chipstream import ChipStream

# XGM instance index
XGM_INDEX = 0
# Output sampling rate settings
SAMPLING_RATE = 44100
SAMPLING_CHUNK_SIZE = 4096

# Sound device init (signed 16bit)
pygame.mixer.pre_init(frequency=SAMPLING_RATE, size=-16, channels=2, buffer=SAMPLING_CHUNK_SIZE)
pygame.init()

# Create Wasm instance
chip_stream = ChipStream()

# Setup XGM
header, gd3 = chip_stream.create_xgm_instance(XGM_INDEX, "./vgm/sor2.xgm", SAMPLING_RATE, SAMPLING_CHUNK_SIZE)
# Print XGM meta
print(header)
print(gd3)

# Play
while chip_stream.xgm_play(XGM_INDEX) == 0:
    # Get sampling referance
    s16le = chip_stream.xgm_get_sampling_ref(XGM_INDEX)
    # Sounds
    sample = pygame.mixer.Sound(buffer=s16le)
    pygame.mixer.Sound.play(sample)
    # Wait pygame mixer
    while pygame.mixer.get_busy() == True:
        pass

# PyGame quit
pygame.quit()

# Drop instance
chip_stream.drop_xgm_instance(XGM_INDEX)
