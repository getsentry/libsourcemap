# -*- coding: utf-8 -*-
from libsourcemap import View

from testutils import get_fixtures


def test_stacktrace():
    _, min_source, min_map = get_fixtures('traceback', with_minified=True)
    view = View.from_json(min_map)

    stacktrace = [
        (0, 63, u'e', 'onFailure'),
        (0, 135, 'r', 'invoke'),
        (0, 182, 'i', 'test'),
        (0, 244, 'nonexisting', None),
    ]

    for line, col, minified, match in stacktrace:
        rv = view.get_original_function_name(line, col, minified, min_source)
        assert rv == match


def test_unicode_stacktrace():
    _, min_source, min_map = get_fixtures('traceback-unicode',
                                          with_minified=True)
    view = View.from_json(min_map)

    stacktrace = [
        (0, 63, u'e', 'onFailure'),
        (0, 135, 'r', 'invoke'),
        (0, 191, 'i', u'ÿ'),
        (0, 244, 'nonexisting', None),
    ]

    for line, col, minified, match in stacktrace:
        rv = view.get_original_function_name(line, col, minified, min_source)
        assert rv == match
