import threading
import libsourcemap


dtors_debug = threading.local()


def dtor_debug_callback(self, error):
    try:
        dtors_debug.set.add(error)
    except AttributeError:
        pass


def pytest_runtest_setup(item):
    dtors_debug.set = set()


def pytest_runtest_teardown(item):
    assert not dtors_debug.set


libsourcemap.highlevel.dtor_debug_callback = dtor_debug_callback
