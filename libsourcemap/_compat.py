import sys


PY2 = sys.version_info[0] == 2

if PY2:
    text_type = unicode

    def implements_to_string(cls):
        cls.__unicode__ = cls.__str__
        cls.__str__ = lambda x: unicode(x).encode('utf-8')
        return cls
else:
    text_type = str
    implements_to_string = lambda x: x


def to_bytes(x):
    if isinstance(x, text_type):
        return x.encode('utf-8')
    if not isinstance(x, bytes):
        raise TypeError('Bytes or string expected')
    return x
