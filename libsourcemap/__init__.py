import os

from contextlib import contextmanager
from collections import namedtuple

from ._sourcemapnative import ffi as _ffi
from ._compat import to_bytes, implements_to_string


_lib = _ffi.dlopen(os.path.join(os.path.dirname(__file__), '_libsourcemap.so'))


Token = namedtuple('Token', ['src_line', 'src_col', 'src_id', 'src', 'name'])


@implements_to_string
class SourcemapError(Exception):

    def __init__(self, message):
        self.message = message

    def __str__(self):
        return self.message.encode('utf-8')


@contextmanager
def capture_err():
    err = _ffi.new('char **')
    def check(rv):
        if rv:
            return rv
        try:
            exc = SourcemapError(_ffi.string(err[0]).decode('utf-8', 'replace'))
        finally:
            _lib.lsm_buffer_free(err[0])
        raise exc
    try:
        yield err, check
    finally:
        pass


class View(object):

    def __init__(self):
        raise TypeError('Cannot instanciate views')

    @staticmethod
    def from_json(buffer):
        buffer = to_bytes(buffer)
        with capture_err() as (err_out, check):
            return View._from_ptr(check(_lib.lsm_view_from_json(
                buffer, len(buffer), err_out)))

    @staticmethod
    def from_memdb(buffer):
        buffer = to_bytes(buffer)
        with capture_err() as (err_out, check):
            return View._from_ptr(check(_lib.lsm_view_from_memdb(
                buffer, len(buffer), err_out)))

    @staticmethod
    def _from_ptr(ptr):
        rv = object.__new__(View)
        rv._ptr = ptr
        return rv

    def dump_memdb(self):
        len_out = _ffi.new('unsigned int *')
        buf = _lib.lsm_view_dump_memdb(self._ptr, len_out)
        try:
            rv = _ffi.unpack(buf, len_out[0])
        finally:
            _lib.lsm_buffer_free(buf)
        return rv

    def lookup_token(self, line, col):
        src_line_out = _ffi.new('unsigned int *')
        src_col_out = _ffi.new('unsigned int *')
        name_out = _ffi.new('char **')
        src_out = _ffi.new('char **')
        src_id_out = _ffi.new('unsigned int *')
        if _lib.lsm_view_lookup_token(self._ptr, line, col, src_line_out,
                                      src_col_out, name_out, src_out,
                                      src_id_out):
            return Token(
                src_line_out[0],
                src_col_out[0],
                src_id_out[0],
                _ffi.string(src_out[0]).decode('utf-8', 'replace'),
                _ffi.string(name_out[0]).decode('utf-8', 'replace')
                    if name_out[0] else None
            )

    def get_source_contents(self, src_id):
        len_out = _ffi.new('unsigned int *')
        rv = _lib.lsm_view_get_source_contents(self._ptr, src_id, len_out)
        if rv:
            return _ffi.unpack(rv, len_out[0])

    def __del__(self):
        try:
            if self._ptr:
                _lib.lsm_view_free(self._ptr)
            self._ptr = None
        except Exception:
            pass
