from libsourcemap import View


def get_fixtures(base):
    with open('tests/fixtures/%s.js' % base, 'rb') as f:
        source = f.read()
    with open('tests/fixtures/%s.min.js' % base, 'rb') as f:
        minified = f.read()
    with open('tests/fixtures/%s.min.map' % base, 'rb') as f:
        min_map = f.read()
    return source, minified, min_map


def test_jquery():
    source, minified, min_map = get_fixtures('jquery')

    source_lines = source.splitlines()

    index = View.from_json(min_map)

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


def test_coolstuff():
    source, minified, min_map = get_fixtures('coolstuff')

    source_lines = source.splitlines()

    index = View.from_json(min_map)

    for token in index:
        if token.name is None:
            continue

        source_line = source_lines[token.src_line]
        start = token.src_col
        end = start + len(token.name)
        substring = source_line[start:end]
        assert token.name == substring


def test_unicode_names():
    _, _, min_map = get_fixtures('unicode')

    # This shouldn't blow up
    View.from_json(min_map)
