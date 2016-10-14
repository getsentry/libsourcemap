#ifndef LIBSOURCEMAP_H_INCLUDED
#define LIBSOURCEMAP_H_INCLUDED

typedef void lsm_view_t;

lsm_view_t *lsm_view_from_json(char *bytes, unsigned int len, char **err_out);
lsm_view_t *lsm_view_from_memdb(char *bytes, unsigned int len, char **err_out);
void lsm_view_free(lsm_view_t *view);

int lsm_view_lookup_token(const lsm_view_t *view, unsigned int line,
                          unsigned int col, unsigned int *src_line_out,
                          unsigned int *src_col_out,
                          const char **name_out,
                          const char **src_out,
                          unsigned int *src_it_out);
const char *lsm_view_get_source_contents(const lsm_view_t *view,
                                         unsigned int src_id,
                                         unsigned int *len_out);
char *lsm_view_dump_memdb(const lsm_view_t *view,
                          unsigned int *len_out);

void lsm_buffer_free(char *buf);

#endif
