import tempfile
from libsourcemap import View

from testutils import get_fixtures, verify_index, verify_token_equivalence, \
    verify_token_search


def test_jquery_mmap():
    source, min_map = get_fixtures('jquery')
    index = View.from_json(min_map)

    with tempfile.NamedTemporaryFile() as f:
        f.write(index.dump_memdb())
        f.flush()

        mem_index = View.from_memdb_file(f.name)
        verify_index(mem_index, source)
        verify_token_equivalence(index, mem_index)
        verify_token_search(mem_index)
