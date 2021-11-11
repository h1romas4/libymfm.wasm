# -*- coding: utf-8 -*-
from wasm.chipstream import ChipStream

# create Wasm instance
chip_stream = ChipStream()

# Setup VGM
chip_stream.create_vgm_instance(0, "./vgm/ym2612.vgm", 44100, 735)

# Play 1 frame (735 sample)
chip_stream.vgm_play(0)
