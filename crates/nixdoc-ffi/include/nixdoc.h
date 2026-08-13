#ifndef NIXDOC_H
#define NIXDOC_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

enum NixdocStatus {
  NIXDOC_SUCCESS = 0,
  NIXDOC_ERROR_PARSE = 1,
  NIXDOC_ERROR_NULL = 2,
  NIXDOC_ERROR_PANIC = 3,
};

typedef struct NixdocDocComment NixdocDocComment;

typedef struct NixdocStringArray {
  char **data;
  size_t len;
} NixdocStringArray;

const char *nixdoc_version(void);
int nixdoc_parse(const char *input);
int nixdoc_parse_into(const char *input, NixdocDocComment **out_doc);
void nixdoc_free(NixdocDocComment *doc);
bool nixdoc_is_doc_comment(const char *input);
/* Returns schema-versioned JSON; release it with nixdoc_free_string. */
char *nixdoc_extract_json(const char *input);
char *nixdoc_title(const NixdocDocComment *doc);
char *nixdoc_description(const NixdocDocComment *doc);
char *nixdoc_type_sig(const NixdocDocComment *doc);
bool nixdoc_is_deprecated(const NixdocDocComment *doc);
char *nixdoc_deprecation_notice(const NixdocDocComment *doc);
NixdocStringArray *nixdoc_arguments(const NixdocDocComment *doc);
NixdocStringArray *nixdoc_examples(const NixdocDocComment *doc);
NixdocStringArray *nixdoc_notes(const NixdocDocComment *doc);
NixdocStringArray *nixdoc_warnings(const NixdocDocComment *doc);
void nixdoc_free_string(char *string);
void nixdoc_free_string_array(NixdocStringArray *array);

#ifdef __cplusplus
}
#endif

#endif
