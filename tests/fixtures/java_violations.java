// expect: no-sysout, no-empty-catch, no-string-equality
// expect: todo-comment, no-empty-function

// TODO: fix
public class JavaViolations {
    void sysout() {
        System.out.println("hello");
    }

    void emptyCatch() {
        try {
            int x = 1;
        } catch (Exception e) {
        }
    }

    void stringEq(String a) {
        if ("hello" == a) {}
    }

    void equalsNull(Object obj) {
        obj.equals(null);
    }

    void empty() {}
}
