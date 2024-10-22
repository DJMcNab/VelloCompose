#include <jni.h>

extern "C" void linker_trick_rust();

extern "C" void linker_trick_cpp() {
    linker_trick_rust();
}