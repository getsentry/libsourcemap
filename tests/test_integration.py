from libsourcemap import View, Index, from_json, IndexedSourceMap

from testutils import get_fixtures, verify_index, verify_token_equivalence, \
    verify_token_search


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
    verify_token_search(index)


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


def test_unified_index_loading():
    with open('tests/fixtures/indexed.sourcemap.js', 'rb') as f:
        index_map = f.read()
    assert isinstance(from_json(index_map), View)
    assert isinstance(from_json(index_map, auto_flatten=False,
                                raise_for_index=False), Index)
    try:
        from_json(index_map, auto_flatten=False)
    except IndexedSourceMap as e:
        assert isinstance(e.index, Index)
    else:
        raise RuntimeError('Expectd an exception')


def test_unified_map_loading():
    with open('tests/fixtures/jquery.min.map', 'rb') as f:
        normal_map = f.read()
    assert isinstance(from_json(normal_map), View)


def test_source_iteration():
    source, min_map = get_fixtures('react-dom')
    index = View.from_json(min_map)
    mem_index = View.from_memdb(index.dump_memdb())
    assert list(index.iter_sources()) == [(0, u'react-dom.js')]
    assert list(mem_index.iter_sources()) == [(0, u'react-dom.js')]


def test_source_access():
    source, min_map = get_fixtures('react-dom-full')
    index = View.from_json(min_map)
    assert index.get_source_contents(0) is not None
    assert index.has_source_contents(0)
    assert index.get_source_contents(1) is None
    assert not index.has_source_contents(1)

    mem_index = View.from_memdb(index.dump_memdb())
    assert mem_index.get_source_contents(0) is not None
    assert mem_index.has_source_contents(0)
    assert mem_index.get_source_contents(1) is None
    assert not mem_index.has_source_contents(1)


def test_memdb_dumping():
    source, min_map = get_fixtures('react-dom-full')
    index = View.from_json(min_map)
    full_mem_index = View.from_memdb(index.dump_memdb())
    nosource_mem_index = View.from_memdb(index.dump_memdb(
        with_source_contents=False))
    nonames_mem_index = View.from_memdb(index.dump_memdb(
        with_names=False))

    for t1, t2, t3 in zip(full_mem_index,
                          nosource_mem_index,
                          nonames_mem_index):
        if t1.name is None:
            continue
        assert t1.name == t2.name
        assert t3.name is None

    assert full_mem_index.get_source_contents(0) is not None
    assert full_mem_index.get_source_contents(0) == \
        nonames_mem_index.get_source_contents(0)
    assert nosource_mem_index.get_source_contents(0) is None
