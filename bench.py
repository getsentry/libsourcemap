import os
import sys
import time

from tempfile import NamedTemporaryFile
from contextlib import contextmanager

from libsourcemap import View


RUNS = 50


def timed_func(f):
    def newf(*args, **kwargs):
        start = time.time()
        sys.stdout.write('%s ... ' % f.__name__)
        sys.stdout.flush()
        try:
            for x in xrange(RUNS):
                rv = f(*args, **kwargs)
        finally:
            sys.stdout.write('%.2fms\n' % ((time.time() - start) * 1000 / RUNS))
            sys.stdout.flush()
        return rv
    return newf


@timed_func
def parse_json(filename):
    with open(filename, 'rb') as f:
        return View.from_json(f.read())


@timed_func
def dump_memdb(view, filename):
    with open(filename, 'wb') as f:
        f.write(view.dump_memdb())


@timed_func
def load_memdb(filename):
    with open(filename) as f:
        return View.from_memdb(f.read())


@timed_func
def load_mmap(filename):
    return View.from_memdb_file(filename)


@timed_func
def lookup_token(view):
    view.lookup_token(1, 1242)
    view.lookup_token(1, 3242)
    view.lookup_token(1, 442)
    view.lookup_token(1, 3242)
    view.lookup_token(2, 5242)


def main():
    path = os.path.abspath(sys.argv[1])

    view = parse_json(path)
    lookup_token(view)

    with NamedTemporaryFile() as f:
        dump_memdb(view, f.name)
        memview = load_memdb(f.name)
        lookup_token(memview)
        memview = load_mmap(f.name)
        lookup_token(memview)


if __name__ == '__main__':
    main()
