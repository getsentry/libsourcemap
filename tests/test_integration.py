from libsourcemap import View

from testutils import get_fixtures, verify_index, verify_token_equivalence


def test_jquery():
    source, min_map = get_fixtures('jquery')
    index = View.from_json(min_map)
    verify_index(index, source)


def test_jquery_memdb():
    source, min_map = get_fixtures('jquery')
    index = View.from_json(min_map)
    mem_index = View.from_memdb(index.dump_memdb())
    verify_index(mem_index, source)
    verify_token_equivalence(index, mem_index)


def test_coolstuff():
    source, min_map = get_fixtures('coolstuff')
    index = View.from_json(min_map)
    verify_index(index, source)


def test_coolstuff_memdb():
    source, min_map = get_fixtures('coolstuff')
    index = View.from_json(min_map)
    mem_index = View.from_memdb(index.dump_memdb())
    verify_index(mem_index, source)
    verify_token_equivalence(index, mem_index)


def test_unicode_names():
    source, min_map = get_fixtures('unicode')
    index = View.from_json(min_map)
    verify_index(index, source)


def test_unicode_names_memdb():
    source, min_map = get_fixtures('unicode')
    index = View.from_json(min_map)
    mem_index = View.from_memdb(index.dump_memdb())
    verify_index(mem_index, source)
    verify_token_equivalence(index, mem_index)


def verify_react_token_search(index):
    # One known token
    react_token = index.lookup_token(0, 319)
    assert react_token.dst_line == 0
    assert react_token.dst_col == 319
    assert react_token.src_line == 39
    assert react_token.src_col == 12
    assert react_token.src_id == 0
    assert react_token.src == 'react-dom.js'
    assert react_token.name == 'React'

    for idx, token in enumerate(index):
        if not token.name:
            continue
        try:
            next_token = index[idx + 1]
            rng = range(token.dst_col, next_token.dst_col)
        except LookupError:
            rng = (token.dst_col,)
        for col in rng:
            token_match = index.lookup_token(token.dst_line, col)
            assert token_match == token


def test_react_dom():
    source, min_map = get_fixtures('react-dom')
    index = View.from_json(min_map)
    verify_index(index, source)
    verify_react_token_search(index)


def test_react_dom_memdb():
    source, min_map = get_fixtures('react-dom')
    index = View.from_json(min_map)
    mem_index = View.from_memdb(index.dump_memdb())
    verify_index(mem_index, source)
    verify_react_token_search(mem_index)
    verify_token_equivalence(index, mem_index)
