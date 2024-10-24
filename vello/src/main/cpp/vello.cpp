#include <jni.h>

// This cpp file exists for two reasons:
// 1) When creating new JNI methods, Android studio will automatically provide the needed name
// in this file. This makes development easier.
// 2) The following two items are needed so that the Rust generated static library is included
// in the created shared library. I do NOT have a strong intuition for why this is necessary,
// and would prefer that it were not.

extern "C" void linker_trick_rust();

extern "C" void linker_trick_cpp() {
    linker_trick_rust();
}