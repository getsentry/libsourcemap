import gc
from libsourcemap import highlevel
from libsourcemap.highlevel import silentdtor


def test_dtors():
    original_dtor_debug_callback = highlevel.dtor_debug_callback
    try:
        failed = []
        def dtor_debug_callback(self, error):
            failed.append(error)
        highlevel.dtor_debug_callback = dtor_debug_callback

        class Foo(object):
            # This dtor is not actually silent, see the conftest.py
            @silentdtor
            def __del__(self):
                raise Exception('Foo')

        try:
            x = Foo()
            del x
            gc.collect()
        except Exception as e:
            pass
        assert len(failed) == 1
        assert failed[0].args == ('Foo',)
    finally:
        highlevel.dtor_debug_callback = original_dtor_debug_callback
