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


class BadJson(SourceMapError):
    """Raised if bad JSON data was encountered."""


class UnsupportedMemDbVersion(SourceMapError):
    """Raised if an unsupported memdb is loaded."""


special_errors = {
    2: IndexedSourceMap,
    3: BadJson,
    4: UnsupportedMemDbVersion,
}
