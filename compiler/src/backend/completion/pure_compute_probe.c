/* This translation unit intentionally includes and links no completion
 * runtime.  The build target checks its symbol table so a pure-compute binary
 * remains evidence of the zero-link boundary rather than merely taking an
 * unused initialization branch. */

static unsigned triangular(unsigned value) {
    return value * (value + 1u) / 2u;
}

int main(void) {
    return triangular(19u) == 190u ? 0 : 1;
}
