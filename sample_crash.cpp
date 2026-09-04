#include <iostream>

int main() {
    int value = 42;
    int *ptr = &value;

    std::cout << "about to succeed...\n";
    *ptr = 99;
    std::cout << "value is " << *ptr << "\n";
    return 0;
}
