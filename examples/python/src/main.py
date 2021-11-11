# -*- coding: utf-8 -*-
import pygame
from wasm.chipstream import ChipStream

# VGM instance index
VGM_INDEX = 0
SAMPLING_RATE = 44100
SAMPLING_CHUNK_SIZE = 735

# Sound device init (signed 16bit)
pygame.mixer.pre_init(frequency=SAMPLING_RATE, size=-16, channels=2, buffer=SAMPLING_CHUNK_SIZE)
pygame.init()

# create Wasm instance
chip_stream = ChipStream()

# Setup VGM (sampling rate: 44100, sample chunk size: 735 = 44100/60 fps)
chip_stream.create_vgm_instance(VGM_INDEX, "./vgm/ym2612.vgm", SAMPLING_RATE, SAMPLING_CHUNK_SIZE)

# Play 1 frame (735 sample)
chip_stream.vgm_play(VGM_INDEX)

# Get sampling referance
s16le = chip_stream.vgm_get_sampling_ref(VGM_INDEX)

# Sounds
sample = pygame.mixer.Sound(buffer=s16le)
pygame.mixer.Sound.play(sample)

# Return
input()

# PyGame quit
pygame.quit()

# Drop instance
chip_stream.drop_vgm_instance(VGM_INDEX)
