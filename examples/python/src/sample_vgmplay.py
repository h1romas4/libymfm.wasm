# -*- coding: utf-8 -*-
# license:BSD-3-Clause
#
# VGM play example
#
import pygame
from wasm.chipstream import ChipStream

# VGM instance index
VGM_INDEX = 0
SAMPLING_RATE = 44100
SAMPLING_CHUNK_SIZE = 4096

# Sound device init (signed 16bit)
pygame.mixer.pre_init(frequency=SAMPLING_RATE, size=-16, channels=2, buffer=SAMPLING_CHUNK_SIZE)
pygame.init()

# create Wasm instance
chip_stream = ChipStream()

# Setup VGM
chip_stream.create_vgm_instance(VGM_INDEX, "./vgm/ym2612.vgm", SAMPLING_RATE, SAMPLING_CHUNK_SIZE)

# Play
while chip_stream.vgm_play(VGM_INDEX) == 0:
    # Get sampling referance
    s16le = chip_stream.vgm_get_sampling_ref(VGM_INDEX)
    # Sounds
    sample = pygame.mixer.Sound(buffer=s16le)
    pygame.mixer.Sound.play(sample)
    # Wait pygame mixer
    while pygame.mixer.get_busy() == True:
        pass

# PyGame quit
pygame.quit()

# Drop instance
chip_stream.drop_vgm_instance(VGM_INDEX)
