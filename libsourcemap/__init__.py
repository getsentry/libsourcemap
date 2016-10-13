import os

from ._sourcemapnative import ffi


lib = ffi.dlopen(os.path.join(os.path.dirname(__file__), '_libsourcemap.so'))
