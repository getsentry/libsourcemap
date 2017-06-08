import os

from contextlib import contextmanager
from collections import namedtuple

from ._sourcemapnative import ffi as _ffi
from ._compat import to_bytes, xrange, NULL_BYTE
from .exceptions import SourceMapError, IndexedSourceMap, special_errors


dtor_debug_callback = None

_lib = _ffi.dlopen(os.path.join(os.path.dirname(__file__), '_libsourcemap.so'))


Token = namedtuple('Token', ['dst_line', 'dst_col', 'src', 'src_line',
                             'src_col', 'src_id', 'name'])


def silentdtor(orig):
    def __del__(self):
        try:
            orig(self)
        except Exception as err:
            if dtor_debug_callback is not None:
                dtor_debug_callback(self, err)
    return __del__


def rustcall(func, *args):
    err = _ffi.new('lsm_error_t *')
    rv = func(*(args + (err,)))
    if not err[0].failed:
        return rv
    try:
        cls = special_errors.get(err[0].code, SourceMapError)
        exc = cls(_ffi.string(err[0].message).decode('utf-8', 'replace'))
    finally:
        _lib.lsm_buffer_free(err[0].message)
    raise exc


rustcall(_lib.lsm_init)


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


def from_json(buffer, auto_flatten=True, raise_for_index=True):
    """Parses a JSON string into either a view or an index.  If auto flatten
    is enabled a sourcemap index that does not contain external references is
    automatically flattened into a view.  By default if an index would be
    returned an `IndexedSourceMap` error is raised instead which holds the
    index.
    """
    buffer = to_bytes(buffer)

    view_out = _ffi.new('lsm_view_t **')
    index_out = _ffi.new('lsm_index_t **')

    buffer = to_bytes(buffer)
    rv = rustcall(_lib.lsm_view_or_index_from_json,
        buffer, len(buffer), view_out, index_out)
    if rv == 1:
        return View._from_ptr(view_out[0])
    elif rv == 2:
        index = Index._from_ptr(index_out[0])
        if auto_flatten and index.can_flatten:
            return index.into_view()
        if raise_for_index:
            raise IndexedSourceMap('Unexpected source map index',
                                   index=index)
        return index
    else:
        raise AssertionError('Unknown response from C ABI (%r)' % rv)


class View(object):
    """Provides a view of a sourcemap.  This can come from two sources:

    * real JSON sourcemaps.  This currently only loads sourcemaps that are
      not indexed.  An indexed sourcemap will raise a `SourcemapError`.
    * preprocessed MemDB sourcemaps.  These are slightly larger (uncompressed)
      than a real sourcemap but can be seeked in without parsing.  This is the
      preferred format for caching.
    """

    def __init__(self):
        raise TypeError('Cannot instantiate views')

    @staticmethod
    def from_json(buffer):
        """Creates a sourcemap view from a JSON string."""
        buffer = to_bytes(buffer)
        return View._from_ptr(rustcall(_lib.lsm_view_from_json,
            buffer, len(buffer)))

    @staticmethod
    def from_memdb(buffer):
        """Creates a sourcemap view from MemDB bytes."""
        buffer = to_bytes(buffer)
        return View._from_ptr(rustcall(_lib.lsm_view_from_memdb,
            buffer, len(buffer)))

    @staticmethod
    def from_memdb_file(path):
        """Creates a sourcemap view from MemDB at a given file."""
        path = to_bytes(path)
        return View._from_ptr(rustcall(_lib.lsm_view_from_memdb_file, path))

    @staticmethod
    def _from_ptr(ptr):
        rv = object.__new__(View)
        rv._ptr = ptr
        return rv

    def _get_ptr(self):
        if not self._ptr:
            raise RuntimeError('View is closed')
        return self._ptr

    def dump_memdb(self, with_source_contents=True, with_names=True):
        """Dumps a sourcemap in MemDB format into bytes."""
        len_out = _ffi.new('unsigned int *')
        buf = rustcall(_lib.lsm_view_dump_memdb,
            self._get_ptr(), len_out,
            with_source_contents, with_names)
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
        if rustcall(_lib.lsm_view_lookup_token, self._get_ptr(),
                    line, col, tok_out):
            return convert_token(tok_out[0])

    def get_source_contents(self, src_id):
        """Given a source ID this returns the embedded sourcecode if there is.
        The sourcecode is returned as UTF-8 bytes for more efficient processing.
        """
        len_out = _ffi.new('unsigned int *')
        must_free = _ffi.new('int *')
        rv = rustcall(_lib.lsm_view_get_source_contents,
                      self._get_ptr(), src_id, len_out, must_free)
        if rv:
            try:
                return _ffi.unpack(rv, len_out[0])
            finally:
                if must_free[0]:
                    _lib.lsm_buffer_free(rv)

    def has_source_contents(self, src_id):
        """Checks if some sources exist."""
        return bool(rustcall(_lib.lsm_view_has_source_contents,
                             self._get_ptr(), src_id))

    def get_source_name(self, src_id):
        """Returns the name of the given source."""
        len_out = _ffi.new('unsigned int *')
        rv = rustcall(_lib.lsm_view_get_source_name,
                      self._get_ptr(), src_id, len_out)
        if rv:
            return decode_rust_str(rv, len_out[0])

    def get_source_count(self):
        """Returns the number of sources."""
        return rustcall(_lib.lsm_view_get_source_count,
                        self._get_ptr())

    def iter_sources(self):
        """Iterates over all source names and IDs."""
        for src_id in xrange(self.get_source_count()):
            yield src_id, self.get_source_name(src_id)

    def __getitem__(self, idx):
        """Returns a token with a given index."""
        tok_out = _ffi.new('lsm_token_t *')
        if rustcall(_lib.lsm_view_get_token, self._get_ptr(), idx, tok_out):
            return convert_token(tok_out[0])
        raise IndexError(idx)

    def __len__(self):
        return rustcall(_lib.lsm_view_get_token_count, self._get_ptr())

    def __iter__(self):
        for idx in xrange(len(self)):
            yield self[idx]

    @silentdtor
    def __del__(self):
        if self._ptr:
            _lib.lsm_view_free(self._ptr)
        self._ptr = None


class Index(object):

    def __init__(self):
        raise TypeError('Cannot instantiate indexes')

    @staticmethod
    def from_json(buffer):
        """Creates an index from a JSON string."""
        buffer = to_bytes(buffer)
        return Index._from_ptr(rustcall(_lib.lsm_index_from_json,
            buffer, len(buffer)))

    @staticmethod
    def _from_ptr(ptr):
        rv = object.__new__(Index)
        rv._ptr = ptr
        return rv

    def _get_ptr(self):
        if not self._ptr:
            raise RuntimeError('Index is closed')
        return self._ptr

    @property
    def can_flatten(self):
        """True if the index does not contain external references."""
        return rustcall(_lib.lsm_index_can_flatten, self._get_ptr()) == 1

    def into_view(self):
        """Converts the index into a view"""
        try:
            return View._from_ptr(rustcall(_lib.lsm_index_into_view,
                self._get_ptr()))
        finally:
            self._ptr = None

    @silentdtor
    def __del__(self):
        if self._ptr:
            _lib.lsm_index_free(self._ptr)
        self._ptr = None


class ProguardView(object):

    def __init__(self):
        raise TypeError('Cannot instantiate proguard views')

    def _get_ptr(self):
        if not self._ptr:
            raise RuntimeError('View is closed')
        return self._ptr

    @staticmethod
    def from_bytes(buffer):
        """Creates a sourcemap view from a JSON string."""
        buffer = to_bytes(buffer)
        return ProguardView._from_ptr(rustcall(
            _lib.lsm_proguard_mapping_from_bytes,
            buffer, len(buffer)))

    @staticmethod
    def from_path(filename):
        """Creates a sourcemap view from a file path."""
        filename = to_bytes(filename)
        if NULL_BYTE in filename:
            raise ValueError('null byte in path')
        return ProguardView._from_ptr(rustcall(
            _lib.lsm_proguard_mapping_from_path,
            filename + b'\x00'))

    @property
    def has_line_info(self):
        """Returns true if the file has line information."""
        return bool(rustcall(
            _lib.lsm_proguard_mapping_has_line_info, self._get_ptr()))

    def lookup(self, dotted_path, lineno=None):
        """Given a dotted path in the format ``class_name`` or
        ``class_name:method_name`` this performs an alias lookup.  For
        methods the line number must be supplied or the result is
        unreliable.
        """
        rv = None
        try:
            rv = rustcall(
                _lib.lsm_proguard_mapping_convert_dotted_path,
                self._get_ptr(),
                dotted_path.encode('utf-8'), lineno or 0)
            return _ffi.string(rv).decode('utf-8', 'replace')
        finally:
            if rv is not None:
                _lib.lsm_buffer_free(rv)

    @staticmethod
    def _from_ptr(ptr):
        rv = object.__new__(ProguardView)
        rv._ptr = ptr
        return rv

    @silentdtor
    def __del__(self):
        if self._ptr:
            _lib.lsm_proguard_mapping_free(self._ptr)
        self._ptr = None
