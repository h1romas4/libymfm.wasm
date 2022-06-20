# -*- coding: utf-8 -*-
# license:BSD-3-Clause
import os
import json
from enum import Enum
from wasmer import engine, wasi, Store, Module, Instance
from wasmer_compiler_cranelift import Compiler

class SoundChipType(Enum):
    YM2149 = 0
    YM2151 = 1
    YM2203 = 2
    YM2413 = 3
    YM2608 = 4
    YM2610 = 5
    YM2612 = 6
    YM3526 = 7
    Y8950 = 8
    YM3812 = 9
    YMF262 = 10
    YMF278B = 11
    SEGAPSG = 12
    SN76489 = 13
    PWM = 14
    SEGAPCM = 15
    OKIM6258 = 16
    C140 = 17

class ChipStream:
    def __init__(self):
        """
        Constructor
        """
        # Let's get the `wasi.wasm` bytes!
        __dir__ = os.path.dirname(os.path.realpath(__file__))
        wasm_bytes = open(__dir__ + '/libymfm.wasm', 'rb').read()
        # Create a store.
        store = Store(engine.JIT(Compiler))
        # Let's compile the Wasm module, as usual.
        module = Module(store, wasm_bytes)
        # Get WASI version (wasi_snapshot_preview1)
        wasi_version = wasi.get_version(module, strict=False)
        # Set WASI enviroment
        wasi_env = \
            wasi.StateBuilder('libymfm'). \
                map_directory('the_host_current_dir', '.'). \
                finalize()
        # Get import objects.
        import_object = wasi_env.generate_import_object(store, wasi_version)
        # Now we can instantiate the module.
        instance = Instance(module, import_object)
        # Set exports in Python instance
        self.wasm = instance.exports

    def create_vgm_instance(self, vgm_instance_id, file_path, output_sampling_rate, output_sample_chunk_size):
        """
        Create VGM Instance in Wasm

        Parameters
        ----------
        vgm_instance_id: int
        file_path: string
        output_sampling_rate: int
        output_sample_chunk_size: int
        """
        # Read VGM file
        vgm_bytes = open(file_path, 'rb').read()
        vgm_length = len(vgm_bytes)
        # Allocate memory in wasm
        memory_id = 0
        self.wasm.memory_alloc(memory_id, vgm_length)
        # Trancefar vgm data
        vgm_ref_pointer = self.wasm.memory_get_ref(memory_id)
        vgm_ref = self.wasm.memory.uint8_view(offset = vgm_ref_pointer)
        vgm_data = bytearray(vgm_bytes)
        for i in range(vgm_length):
            vgm_ref[i] = vgm_data[i]
        # Create VgmPlay instance in Wasm
        self.wasm.vgm_create(vgm_instance_id, output_sampling_rate, output_sample_chunk_size, memory_id)
        # Drop allocate memory in wasm
        self.wasm.memory_drop(memory_id)
        # Set sampling chunk into instance
        self.output_sample_chunk_size = output_sample_chunk_size * 4 # s16le * 2ch
        # Get VGM header JSON
        return json.loads(self.get_wasm_string(self.wasm.vgm_get_header_json(vgm_instance_id))), \
            json.loads(self.get_wasm_string(self.wasm.vgm_get_gd3_json(vgm_instance_id)))

    def create_xgm_instance(self, xgm_instance_id, file_path, output_sampling_rate, output_sample_chunk_size):
        """
        Create XGM Instance in Wasm

        Parameters
        ----------
        xgm_instance_id: int
        file_path: string
        output_sampling_rate: int
        output_sample_chunk_size: int
        """
        # Read VGM file
        xgm_bytes = open(file_path, 'rb').read()
        xgm_length = len(xgm_bytes)
        # Allocate memory in wasm
        memory_id = 0
        self.wasm.memory_alloc(memory_id, xgm_length)
        # Trancefar vgm data
        xgm_ref_pointer = self.wasm.memory_get_ref(memory_id)
        xgm_ref = self.wasm.memory.uint8_view(offset = xgm_ref_pointer)
        xgm_data = bytearray(xgm_bytes)
        for i in range(xgm_length):
            xgm_ref[i] = xgm_data[i]
        # Create VgmPlay instance in Wasm
        self.wasm.xgm_create(xgm_instance_id, output_sampling_rate, output_sample_chunk_size, memory_id)
        # Drop allocate memory in wasm
        self.wasm.memory_drop(memory_id)
        # Set sampling chunk into instance
        self.output_sample_chunk_size = output_sample_chunk_size * 4 # s16le * 2ch
        # Get VGM header JSON
        return json.loads(self.get_wasm_string(self.wasm.xgm_get_header_json(xgm_instance_id))), \
            json.loads(self.get_wasm_string(self.wasm.xgm_get_gd3_json(xgm_instance_id)))

    def get_wasm_string(self, memory_index_id):
        # Create memory view
        memory_view = self.wasm.memory.uint8_view(offset = self.wasm.memory_get_ref(memory_index_id))
        memory_length = self.wasm.memory_get_len(memory_index_id)
        # Copy into Python array
        wasm_string = [0] * memory_length
        for i in range(memory_length):
            wasm_string[i] = memory_view[i]
        # Memory drop
        self.wasm.memory_drop(memory_index_id)
        # UTF-8 string
        return bytearray(wasm_string).decode()

    def vgm_play(self, vgm_instance_id):
        """
        Play VGM

        Parameters
        ----------
        vgm_instance_id: int
        """
        return self.wasm.vgm_play(vgm_instance_id)

    def vgm_get_sampling_ref(self, vgm_instance_id):
        """
        Get sampling s16le

        Parameters
        ----------
        vgm_instance_id: int

        Returns
        ----------
        memoryview
        """
        ref = self.wasm.vgm_get_sampling_s16le_ref(vgm_instance_id)
        memory = bytearray(self.wasm.memory.buffer)
        return memoryview(memory[ref:ref + self.output_sample_chunk_size])

    def drop_vgm_instance(self, vgm_instance_id):
        """
        Drop VGM instance

        Parameters
        ----------
        vgm_instance_id: int
        """
        self.wasm.vgm_drop(vgm_instance_id)

    def xgm_play(self, xgm_instance_id):
        """
        Play XGM

        Parameters
        ----------
        xgm_instance_id: int
        """
        return self.wasm.xgm_play(xgm_instance_id)

    def xgm_get_sampling_ref(self, xgm_instance_id):
        """
        Get sampling s16le

        Parameters
        ----------
        xgm_instance_id: int

        Returns
        ----------
        memoryview
        """
        ref = self.wasm.xgm_get_sampling_s16le_ref(xgm_instance_id)
        memory = bytearray(self.wasm.memory.buffer)
        return memoryview(memory[ref:ref + self.output_sample_chunk_size])

    def drop_xgm_instance(self, xgm_instance_id):
        """
        Drop VGM instance

        Parameters
        ----------
        xgm_instance_id: int
        """
        self.wasm.xgm_drop(xgm_instance_id)

    def sound_slot_create(self, sound_slot_instance_id, external_tick_rate, output_sampling_rate, output_sample_chunk_size):
        """
        Create sound slot instance in Wasm

        Parameters
        ----------
        sound_slot_instance_id: int
        external_tick_rate: int
        output_sampling_rate: int
        output_sample_chunk_size: int
        """
        self.wasm.sound_slot_create(sound_slot_instance_id, external_tick_rate, output_sampling_rate, output_sample_chunk_size)
        self.sound_slot_output_sample_chunk_size = output_sample_chunk_size * 4 # s16le * 2ch

    def sound_slot_add_sound_device(self, sound_slot_instance_id, sound_chip_type: SoundChipType, number_of, clock):
        """
        Add sound chip to sound slot

        Parameters
        ----------
        sound_slot_instance_id: int
        sound_chip_type: SoundChipType
        number_of: int
        clock: int
        """
        self.wasm.sound_slot_add_sound_device(sound_slot_instance_id, sound_chip_type.value, number_of, clock)

    def sound_slot_write(self, sound_slot_instance_id, sound_chip_type: SoundChipType, sound_chip_index, port, data):
        """
        Write sound chip to command

        Parameters
        ----------
        sound_slot_instance_id: int
        sound_chip_type: SoundChipType
        sound_chip_index: int
        port: int
        data: int
        """
        self.wasm.sound_slot_write(sound_slot_instance_id, sound_chip_type.value, sound_chip_index, port, data)

    def sound_slot_update(self, sound_slot_instance_id, tick_count):
        """
        Update sound slot

        Parameters
        ----------
        sound_slot_instance_id: int
        tick_count: int
        """
        self.wasm.sound_slot_update(sound_slot_instance_id, tick_count)

    def sound_slot_is_stream_filled(self, sound_slot_instance_id):
        """
        Query sampling chunk buffered

        Parameters
        ----------
        sound_slot_instance_id: int
        """
        return self.wasm.sound_slot_is_stream_filled(sound_slot_instance_id)

    def sound_slot_stream(self, sound_slot_instance_id):
        """
        Set sampling to stream

        Parameters
        ----------
        sound_slot_instance_id: int
        """
        self.wasm.sound_slot_stream(sound_slot_instance_id)

    def sound_slot_get_sampling_ref(self, sound_slot_instance_id):
        """
        Get sampling s16le

        Parameters
        ----------
        sound_slot_instance_id: int

        Returns
        ----------
        memoryview
        """
        ref = self.wasm.sound_slot_sampling_s16le_ref(sound_slot_instance_id)
        memory = bytearray(self.wasm.memory.buffer)
        return memoryview(memory[ref:ref + self.sound_slot_output_sample_chunk_size])

    def sound_slot_drop(self, sound_slot_instance_id):
        """
        Drop sound slot instance

        Parameters
        ----------
        sound_slot_instance_id: int
        """
        self.wasm.sound_slot_drop(sound_slot_instance_id)
