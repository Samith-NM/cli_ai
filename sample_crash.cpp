#include <iostream>

int main() {
    int *ptr = nullptr;
    std::cout << "about to crash...\n";
    *ptr = 42;
    return 0;
}
