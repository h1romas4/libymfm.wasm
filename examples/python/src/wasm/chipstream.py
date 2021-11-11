# -*- coding: utf-8 -*-
from wasmer import engine, wasi, Store, Module, ImportObject, Instance
from wasmer_compiler_cranelift import Compiler
import os

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
        # Create VgmPlay instance in Wasm
        self.wasm.vgm_create(vgm_instance_id, output_sampling_rate, output_sample_chunk_size, vgm_length)
        self.output_sample_chunk_size = output_sample_chunk_size * 4 # f32
        # Write VGM data to Wasm memory
        vgm_ref_pointer = self.wasm.vgm_get_seq_data_ref(vgm_instance_id)
        vgm_ref = self.wasm.memory.uint8_view(offset = vgm_ref_pointer)
        vgm_data = bytearray(vgm_bytes)
        for i in range(vgm_length):
            vgm_ref[i] = vgm_data[i]
        # Initialize VgmPlay
        self.wasm.vgm_init(vgm_instance_id)

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
        Play VGM

        Parameters
        ----------
        vgm_instance_id: int

        Returns
        ----------
        (sampling_l, sampling_r): (memory, memory)
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
