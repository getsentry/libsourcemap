import os

from contextlib import contextmanager
from collections import namedtuple

from ._sourcemapnative import ffi as _ffi
from ._compat import to_bytes
from .exceptions import SourceMapError, special_errors


_lib = _ffi.dlopen(os.path.join(os.path.dirname(__file__), '_libsourcemap.so'))


Token = namedtuple('Token', ['dst_line', 'dst_col', 'src', 'src_line',
                             'src_col', 'src_id', 'name'])


@contextmanager
def capture_err():
    err = _ffi.new('lsm_error_t *')
    def check(rv):
        if rv:
            return rv
        try:
            cls = special_errors.get(err[0].code, SourceMapError)
            exc = cls(_ffi.string(err[0].message).decode('utf-8', 'replace'))
        finally:
            _lib.lsm_buffer_free(err[0].message)
        raise exc
    try:
        yield err, check
    finally:
        pass


def decode_rust_str(ptr, len):
    if ptr:
        return _ffi.unpack(ptr, len).decode('utf-8', 'replace')


def convert_token(tok):
    return Token(
        tok.dst_line,
        tok.dst_col,
        decode_rust_str(tok.src, tok.src_len),
        tok.src_line,
        tok.src_col,
        tok.src_id,
        decode_rust_str(tok.name, tok.name_len)
    )


class View(object):
    """Provides a view of a sourcemap.  This can come from two sources:

    * real JSON sourcemaps.  This currently only loads sourcemaps that are
      not indexed.  An indexed sourcemap will raise a `SourcemapError`.
    * preprocessed MemDB sourcemaps.  These are slightly larger (uncompressed)
      than a real sourcemap but can be seeked in without parsing.  This is the
      preferred format for caching.
    """

    def __init__(self):
        raise TypeError('Cannot instanciate views')

    @staticmethod
    def from_json(buffer):
        """Creates a sourcemap view from a JSON string."""
        buffer = to_bytes(buffer)
        with capture_err() as (err_out, check):
            return View._from_ptr(check(_lib.lsm_view_from_json(
                buffer, len(buffer), err_out)))

    @staticmethod
    def from_memdb(buffer):
        """Creates a sourcemap view from MemDB bytes."""
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
        """Dumps a sourcemap in MemDB format into bytes."""
        len_out = _ffi.new('unsigned int *')
        buf = _lib.lsm_view_dump_memdb(self._ptr, len_out)
        try:
            rv = _ffi.unpack(buf, len_out[0])
        finally:
            _lib.lsm_buffer_free(buf)
        return rv

    def lookup_token(self, line, col):
        """Given a minified location, this tries to locate the closest
        token that is a match.  Returns `None` if no match can be found.
        """
        # Silently ignore underflows
        if line < 0 or col < 0:
            return None
        tok_out = _ffi.new('lsm_token_t *')
        if _lib.lsm_view_lookup_token(self._ptr, line, col, tok_out):
            return convert_token(tok_out[0])

    def get_source_contents(self, src_id):
        """Given a source ID this returns the embedded sourcecode if there is.
        The sourcecode is returned as UTF-8 bytes for more efficient processing.
        """
        len_out = _ffi.new('unsigned int *')
        rv = _lib.lsm_view_get_source_contents(self._ptr, src_id, len_out)
        if rv:
            return _ffi.unpack(rv, len_out[0])

    def __getitem__(self, idx):
        """Returns a token with a given index."""
        tok_out = _ffi.new('lsm_token_t *')
        if _lib.lsm_view_get_token(self._ptr, idx, tok_out):
            return convert_token(tok_out[0])
        raise IndexError(idx)

    def __len__(self):
        return _lib.lsm_view_get_token_count(self._ptr)

    def __iter__(self):
        for idx in xrange(len(self)):
            yield self[idx]

    def __del__(self):
        try:
            if self._ptr:
                _lib.lsm_view_free(self._ptr)
            self._ptr = None
        except Exception:
            pass
