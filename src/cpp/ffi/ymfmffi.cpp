#include <stdio.h>

extern "C" {
    int ffi_test(int count) {
        int ans = 0;
        for (size_t i = 0; i < count; i++)
        {
            ans += i;
        }
        return ans;
    }
}
