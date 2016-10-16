from libsourcemap import View


def get_fixtures(base):
    with open('tests/fixtures/%s.js' % base, 'rb') as f:
        source = f.read()
    with open('tests/fixtures/%s.min.map' % base, 'rb') as f:
        min_map = f.read()
    return source, min_map


def verify_index(index, source):
    source_lines = source.decode('utf-8').splitlines()
    for token in index:
        # Ignore tokens that are None.
        # There's no simple way to verify they're correct
        if token.name is None:
            continue
        source_line = source_lines[token.src_line]
        start = token.src_col
        end = start + len(token.name)
        substring = source_line[start:end]

        # jQuery's sourcemap has a few tokens that are identified
        # incorrectly.
        # For example, they have a token for 'embed', and
        # it maps to '"embe', which is wrong. This only happened
        # for a few strings, so we ignore
        if substring[0] == '"':
            continue
        assert token.name == substring


def test_jquery():
    source, min_map = get_fixtures('jquery')
    index = View.from_json(min_map)
    verify_index(index, source)


def test_jquery_memdb():
    source, min_map = get_fixtures('jquery')
    index = View.from_json(min_map)
    mem_index = View.from_memdb(index.dump_memdb())
    verify_index(mem_index, source)


def test_coolstuff():
    source, min_map = get_fixtures('coolstuff')
    index = View.from_json(min_map)
    verify_index(index, source)


def test_coolstuff_memdb():
    source, min_map = get_fixtures('coolstuff')
    index = View.from_json(min_map)
    mem_index = View.from_memdb(index.dump_memdb())
    verify_index(mem_index, source)


def test_unicode_names():
    source, min_map = get_fixtures('unicode')
    index = View.from_json(min_map)
    verify_index(index, source)


def test_unicode_names_memdb():
    source, min_map = get_fixtures('unicode')
    index = View.from_json(min_map)
    mem_index = View.from_memdb(index.dump_memdb())
    verify_index(mem_index, source)


def test_react_dom():
    source, min_map = get_fixtures('react-dom')
    index = View.from_json(min_map)
    verify_index(index, source)


def test_react_dom_memdb():
    source, min_map = get_fixtures('react-dom')
    index = View.from_json(min_map)
    mem_index = View.from_memdb(index.dump_memdb())
    verify_index(mem_index, source)
