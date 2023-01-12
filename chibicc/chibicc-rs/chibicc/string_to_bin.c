#include "chibicc.h"
#define static_test_string 1

char *string_to_bin(char* input, size_t output_buf_size) {
  if (static_test_string) {
      input = malloc(100);
      strcpy(input, "int main(int argc, char** argv) { return 0; }\n");
  }
  Token *tok = tokenize_input_string(input);
  tok = preprocess(tok);
  Obj *prog = parse(tok);
  char *bin_buf = malloc(output_buf_size);
  codegen_mem_buf(prog, bin_buf);

  return bin_buf;
}
