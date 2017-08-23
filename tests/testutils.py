try:
    from itertools import izip
except ImportError:
    izip = zip


def get_fixtures(base, with_minified=False):
    with open('tests/fixtures/%s.js' % base, 'rb') as f:
        source = f.read()
    with open('tests/fixtures/%s.min.map' % base, 'rb') as f:
        min_map = f.read()

    if not with_minified:
        return source, min_map

    with open('tests/fixtures/%s.min.js' % base, 'rb') as f:
        minified_source = f.read()
    return source, minified_source, min_map


def verify_index(index, source):
    if isinstance(source, dict):
        sources = dict((k, v.decode('utf-8').splitlines())
                       for k, v in source.iteritems())
    else:
        source_lines = source.decode('utf-8').splitlines()

    def get_source_line(token):
        if isinstance(source, dict):
            return sources[token.src][token.src_line]
        return source_lines[token.src_line]

    for token in index:
        # Ignore tokens that are None.
        # There's no simple way to verify they're correct
        if token.name is None:
            continue
        source_line = get_source_line(token)
        start = token.src_col
        end = start + len(token.name)
        substring = source_line[start:end]

        # jQuery's sourcemap has a few tokens that are identified
        # incorrectly.
        # For example, they have a token for 'embed', and
        # it maps to '"embe', which is wrong. This only happened
        # for a few strings, so we ignore
        if substring[:1] == '"':
            continue

        assert token.name == substring


def verify_token_equivalence(index, mem_index):
    for tok1, tok2 in izip(index, mem_index):
        assert tok1 == tok2


def verify_token_search(index):
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
