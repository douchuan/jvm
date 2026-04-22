import java.util.ArrayList;
import java.util.List;

public class Generics {
    // Generic method
    public static <T> void printItem(T item) {
        System.out.println("Item: " + item);
    }

    // Generic class usage
    public static void main(String[] args) {
        // ArrayList with generic type
        List<String> names = new ArrayList<String>();
        names.add("Alice");
        names.add("Bob");
        names.add("Charlie");

        for (int i = 0; i < names.size(); i++) {
            System.out.println("Name " + i + ": " + names.get(i));
        }

        // List of Integer
        List<Integer> numbers = new ArrayList<Integer>();
        numbers.add(10);
        numbers.add(20);
        numbers.add(30);

        int sum = 0;
        for (int i = 0; i < numbers.size(); i++) {
            sum += numbers.get(i);
        }
        System.out.println("Sum: " + sum);

        // Generic method call
        printItem("Hello from generics");
        printItem(42);
    }
}
