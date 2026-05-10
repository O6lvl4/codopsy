// Realistic C# code with various issues
// TODO: add logging

using System;
using System.Collections.Generic;

class Sample {
    void Process(List<int> items) {
        foreach (var item in items) {
            Console.WriteLine(item);
            if (item > 10) {
                if (item > 20) {
                    Console.WriteLine("big: " + item);
                }
            }
        }
    }

    void Empty() {}

    int Complex(int a, int b, int c, int d, int e) {
        return a + b + c + d + e;
    }
}
