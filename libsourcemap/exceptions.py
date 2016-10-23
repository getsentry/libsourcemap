from ._compat import implements_to_string


@implements_to_string
class SourceMapError(Exception):
    """Raised if something goes wrong with sourcemap processing."""

    def __init__(self, message):
        self.message = message

    def __str__(self):
        return self.message.encode('utf-8')


class IndexedSourceMap(SourceMapError):
    """Raised if a sourcemap is indexed."""

    def __init__(self, message, index=None):
        SourceMapError.__init__(self, message)
        self.index = index


class BadJson(SourceMapError):
    """Raised if bad JSON data was encountered."""


class CannotFlatten(SourceMapError):
    """Raised if an index cannot be flattened into a sourcemap."""


class UnsupportedMemDbVersion(SourceMapError):
    """Raised if an unsupported memdb is loaded."""


class BadIo(SourceMapError):
    """Raised if an IO error happened."""


class MemDbDumpError(SourceMapError):
    """Raised if creating a memdb is not possible."""


class TooManySources(MemDbDumpError):
    """There are too many sources for memdb."""


class TooManyNames(MemDbDumpError):
    """There are too many names for memdb."""


class LocationOverflow(MemDbDumpError):
    """The location information is too large for the memdb format."""


class AlreadyMemDb(MemDbDumpError):
    """Cannot create a memdb from a memdb."""


special_errors = {
    2: IndexedSourceMap,
    3: BadJson,
    4: CannotFlatten,
    5: UnsupportedMemDbVersion,
    6: BadIo,

    20: TooManySources,
    21: TooManyNames,
    22: LocationOverflow,
    23: AlreadyMemDb,
}
