# -*- coding: utf-8 -*-
from wasmer import engine, wasi, Store, Module, ImportObject, Instance
from wasmer_compiler_cranelift import Compiler
import os

# Let's get the `wasi.wasm` bytes!
__dir__ = os.path.dirname(os.path.realpath(__file__))
wasm_bytes = open(__dir__ + '/wasm/libymfm.wasm', 'rb').read()

# Create a store.
store = Store(engine.JIT(Compiler))

# Let's compile the Wasm module, as usual.
module = Module(store, wasm_bytes)

# Get WASI version (wasi_snapshot_preview1)
wasi_version = wasi.get_version(module, strict=False)

# Set WASI env
wasi_env = \
    wasi.StateBuilder('libymfm'). \
        map_directory('the_host_current_dir', '.'). \
        finalize()

# Get import objects.
import_object = wasi_env.generate_import_object(store, wasi_version)

# Now we can instantiate the module.
instance = Instance(module, import_object)

# test
print(instance.exports)
