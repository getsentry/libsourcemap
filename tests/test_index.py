from libsourcemap import Index

from testutils import verify_index


def test_load_index():
    with open('tests/fixtures/indexed.sourcemap.js', 'rb') as f:
        index_map = f.read()
    with open('tests/fixtures/file1.js', 'rb') as f:
        f1 = f.read()
    with open('tests/fixtures/file2.js', 'rb') as f:
        f2 = f.read()

    idx = Index.from_json(index_map)
    view = idx.into_view()
    for token in view:
        print token
    verify_index(view, {
        'file1.js': f1,
        'file2.js': f2,
    })
