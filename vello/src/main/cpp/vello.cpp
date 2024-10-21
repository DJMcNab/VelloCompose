#include <jni.h>
#include <string>

extern "C" {
    void rust_expose_value(char* data, size_t len);
}

extern "C" JNIEXPORT jstring JNICALL
Java_org_linebender_vello_NativeLib_stringFromJNI(
        JNIEnv* env,
        jobject /* this */) {
    char * area = (char*)malloc(100);
    rust_expose_value(area, 100);
    std::string hello;
    hello.append(area);
    free(area);
    return env->NewStringUTF(hello.c_str());
}
