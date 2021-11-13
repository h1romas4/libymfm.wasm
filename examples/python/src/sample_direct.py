# -*- coding: utf-8 -*-
# license:BSD-3-Clause
#
# Direct Sound Chip Access Example
#
import pygame
from wasm.chipstream import ChipStream, SoundChipType

# The Sound slot index 0
SOUND_SLOT_INDEX = 0
# Output sampling rate settings
SAMPLING_RATE = 44100
SAMPLING_CHUNK_SIZE = 4096
# Sound driver tick rate
SOUND_DRIVER_TICK_RATE = 60
# Sound chip clock (*4: temporary hack)
YM2149_CLOCK = 3579545 * 4

# YM2149 tone table
TONE_TABLE = [
    3420,3229,3047,2876,2715,2562,2419,2283,2155,2034,1920,1812, #o1  0〜 11
	1710,1614,1524,1438,1357,1281,1209,1141,1077,1017, 960, 906, #o2 12〜 23
	 855, 807, 762, 719, 679, 641, 605, 571, 539, 508, 480, 453, #o3 24〜 35
	 428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226, #o4 36〜 47
	 214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113, #o5 48〜 59
	 107, 101,  95,  90,  85,  80,  76,  71,  67,  64,  60,  57, #o6 60〜 71
	  53,  50,  48,  45,  42,  40,  38,  36,  34,  32,  30,  28, #o7 72〜 83
	  27,  25,  24,  22,  21,  20,  19,  18,  17,  16,  15,  14, #o8 84〜 95
]

#
# Music sequence data
# [
#   # Channel 1
#   [
#       TONE_NUMBER|COMMAND, WAIT_TICK_COUNT,
#       ...
#   ],
#   # Channel 2
#   [
#       ...
#   ],
#   # Channel 3
#   [
#       ...
#   ],
# ]
MUSIC_SEQUENCE = [
    # Channel 1
    [
        48,  6, 60, 6, 59, 6, 60, 6,
        64,  6, 60, 6, 59, 6, 60, 6,
        48,  6, 60, 6, 58, 6, 60, 6,
        64,  6, 60, 6, 58, 6, 60, 6,
        48,  6, 60, 6, 57, 6, 60, 6,
        64,  6, 60, 6, 57, 6, 60, 6,
        48,  6, 60, 6, 56, 6, 60, 6,
        64,  6, 60, 6, 56, 6, 60, 6,
    ],
    [],
    [],
]

# Sound device init (signed 16bit)
pygame.mixer.pre_init(frequency=SAMPLING_RATE, size=-16, channels=2, buffer=SAMPLING_CHUNK_SIZE)
pygame.init()

# Create Wasm instance
chip_stream = ChipStream()
# Sampling buffer
sampling_buffer = []

# Setup sound slot
chip_stream.sound_slot_create(SOUND_SLOT_INDEX, SOUND_DRIVER_TICK_RATE, SAMPLING_RATE, SAMPLING_CHUNK_SIZE)

# Add "one" YM2149 sound chip in sound slot
chip_stream.sound_slot_add_sound_device(SOUND_SLOT_INDEX, SoundChipType.YM2149, 1, YM2149_CLOCK)

# YM2149 initialize (write reg: 0x7, data: 0b00111000)
chip_stream.sound_slot_write(SOUND_SLOT_INDEX, SoundChipType.YM2149, 0, 0x7, 0b00111000)

# Setup sequence work
seq_index = [0, 0, 0]
wait = [0, 0, 0]
loop_count = 30

# Play loop (1 loop is 1 tick)
while loop_count > 0:
    #
    # Analyze music sequence
    #
    for track in range(0, 3):
        if len(MUSIC_SEQUENCE[track]) <= 0:
            continue
        # Wait 1 tick
        if wait[track] > 0:
            wait[track] -= 1
            continue
        # Get sequence command
        command = MUSIC_SEQUENCE[track][seq_index[track]]
        # Tone
        if 0 <= command < len(TONE_TABLE):
            tone = TONE_TABLE[command]
            # 16bit little endian
            lower = tone & 0xff
            upper = (tone & 0xff00) >> 8
            # Write YM2149
            # Set volume
            chip_stream.sound_slot_write(SOUND_SLOT_INDEX, SoundChipType.YM2149, 0, track + 0x8, 15)
            # Set frequency
            chip_stream.sound_slot_write(SOUND_SLOT_INDEX, SoundChipType.YM2149, 0, track * 2, lower)
            chip_stream.sound_slot_write(SOUND_SLOT_INDEX, SoundChipType.YM2149, 0, track * 2 + 1, upper)
            # Set wait tick
            seq_index[track] += 1
            wait[track] = MUSIC_SEQUENCE[track][seq_index[track]]
        # Next sequence
        seq_index[track] += 1
        # Loop sequence
        if seq_index[track] >= len(MUSIC_SEQUENCE[track]):
            seq_index[track] = 0
            loop_count -= 1
    #
    # Update sound
    #
    # Tick "One" sound slot (SOUND_DRIVER_TICK_RATE = 1/60)
    chip_stream.sound_slot_update(SOUND_SLOT_INDEX, 1)
    # Query sampling chunk filled
    if chip_stream.sound_slot_is_stream_filled(SOUND_SLOT_INDEX) == 1:
        # Stream sound slot
        chip_stream.sound_slot_stream(SOUND_SLOT_INDEX)
        # Get sampling referance
        s16le = chip_stream.sound_slot_get_sampling_ref(SOUND_SLOT_INDEX)
        # Add to the sampling buffer
        sampling_buffer.append(s16le)
    # If the sampling buffer is full, the sound will be played
    if len(sampling_buffer) > 0 and pygame.mixer.get_busy() == False:
        # Remove the first element from the sampling buffer and pack it
        s16le = pygame.mixer.Sound(buffer=sampling_buffer.pop(0))
        # Play!
        pygame.mixer.Sound.play(s16le)

# Drop sound slot
chip_stream.sound_slot_drop(SOUND_SLOT_INDEX)

# PyGame quit
pygame.quit()
