// Realistic Java code with various issues
public class Sample {
    void printStuff() {
        System.out.println("hello");
        System.err.println("error");
    }

    void badCatch() {
        try {
            int x = 1 / 0;
        } catch (Exception e) {
        }
    }

    void stringCompare(String a, String b) {
        if ("hello" == a) {}
        if (a.equals(null)) {}
    }

    void empty() {}

    void nested() {
        try {
            try {
                int x = 1;
            } catch (Exception e) {
                e.printStackTrace();
            }
        } catch (Exception e) {}
    }

    void loopConcat() {
        String result = "";
        for (int i = 0; i < 10; i++) {
            result += "item" + i;
        }
    }
}
