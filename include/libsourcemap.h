#ifndef LIBSOURCEMAP_H_INCLUDED
#define LIBSOURCEMAP_H_INCLUDED

typedef void lsm_view_t;
typedef void lsm_index_t;

typedef struct lsm_token_s {
    unsigned int dst_line;
    unsigned int dst_col;
    unsigned int src_line;
    unsigned int src_col;
    const char *name;
    unsigned int name_len;
    const char *src;
    unsigned int src_len;
    unsigned int src_id;
} lsm_token_t;

typedef struct lsm_error_s {
    char *message;
    int code;
} lsm_error_t;

lsm_view_t *lsm_view_from_json(char *bytes, unsigned int len, lsm_error_t *err);
lsm_view_t *lsm_view_from_memdb(char *bytes, unsigned int len, lsm_error_t *err);
void lsm_view_free(lsm_view_t *view);

unsigned int lsm_view_get_token_count(const lsm_view_t *view);
unsigned int lsm_view_get_token(const lsm_view_t *view, unsigned int idx,
                                lsm_token_t *tok_out);
int lsm_view_lookup_token(const lsm_view_t *view, unsigned int line,
                          unsigned int col, lsm_token_t *tok_out);
unsigned int lsm_view_get_source_count(const lsm_view_t *view);
const char *lsm_view_get_source_contents(const lsm_view_t *view,
                                         unsigned int src_id,
                                         unsigned int *len_out);
const char *lsm_view_get_source_name(const lsm_view_t *view,
                                     unsigned int src_id,
                                     unsigned int *len_out);
char *lsm_view_dump_memdb(const lsm_view_t *view,
                          unsigned int *len_out);

lsm_index_t *lsm_index_from_json(char *bytes, unsigned int len, lsm_error_t *err);
void lsm_index_free(lsm_index_t *index);
int lsm_index_can_flatten(const lsm_index_t *index);
lsm_view_t *lsm_index_into_view(lsm_index_t *index, lsm_error_t *err);

void lsm_buffer_free(char *buf);

#endif
