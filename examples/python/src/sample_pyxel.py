# -*- coding: utf-8 -*-
# license:BSD-3-Clause
#
# Pyxel example
#
import pygame
import pyxel
from wasm.chipstream import ChipStream

class App:
    # The VGM instance
    VGM_INDEX = 0
    # Audio output sampling rate
    SAMPLING_RATE = 44100
    # Audio chunk size (1/60 = 735 sample)
    # The chunk size of PyGame must be a power of 2 (512, 1024...)
    SAMPLING_CHUNK_SIZE = 4096

    def __init__(self):
        # Initialize audio system (pygame SDL)
        pygame.mixer.pre_init(frequency=App.SAMPLING_RATE, size=-16, channels=2, buffer=App.SAMPLING_CHUNK_SIZE)
        pygame.init()
        # Create Wasm instance
        self.chip_stream = ChipStream()
        # Load and initialize music data
        self.chip_stream.create_vgm_instance(App.VGM_INDEX, "./vgm/ym2612.vgm", App.SAMPLING_RATE, App.SAMPLING_CHUNK_SIZE)
        # Create sampling buffer
        self.sampling_buffer = []
        # pyxel sample
        self.moji_x = 46
        self.moji_vx = 1
        # Initialize pyxel
        pyxel.init(160, 120, caption="Hello ChipStream")
        pyxel.run(self.update, self.draw)

    def update(self):
        # If the buffer advance is within 2, create a sampling data
        if len(self.sampling_buffer) < 2:
            # Create a sampling of the chunk size (1024)
            self.chip_stream.vgm_play(App.VGM_INDEX)
            # Get sampling referance
            s16le = self.chip_stream.vgm_get_sampling_ref(App.VGM_INDEX)
            # Add to the sampling buffer
            self.sampling_buffer.append(s16le)
        # pyxel sample
        self.moji_x += self.moji_vx
        if self.moji_x >= 86 or self.moji_x <= 2:
            self.moji_vx *= -1
        if pyxel.btnp(pyxel.KEY_Q):
            # Drop instance
            self.chip_stream.drop_vgm_instance(App.VGM_INDEX)
            # Quit PyGame
            pygame.quit()
            # Quit pyxel
            pyxel.quit()

    def draw(self):
        # If the sampling buffer is full, the sound will be played
        if len(self.sampling_buffer) > 0 and pygame.mixer.get_busy() == False:
            # Remove the first element from the sampling buffer and pack it
            s16le = pygame.mixer.Sound(buffer=self.sampling_buffer.pop(0))
            # Play!
            pygame.mixer.Sound.play(s16le)
        pyxel.cls(0)
        pyxel.text(self.moji_x, 48, "Hello, ChipStream!", pyxel.frame_count % 16)
        pyxel.blt(61, 66, 0, 0, 0, 38, 16)

App()
