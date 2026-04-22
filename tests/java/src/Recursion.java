public class Recursion {
    // Fibonacci
    public static int fib(int n) {
        if (n <= 1) return n;
        return fib(n - 1) + fib(n - 2);
    }

    // Factorial
    public static long factorial(int n) {
        if (n <= 1) return 1;
        return n * factorial(n - 1);
    }

    // Sum of array (recursive)
    public static int sumArray(int[] arr, int index) {
        if (index >= arr.length) return 0;
        return arr[index] + sumArray(arr, index + 1);
    }

    // Binary search (recursive)
    public static int binarySearch(int[] arr, int target, int low, int high) {
        if (low > high) return -1;
        int mid = (low + high) / 2;
        if (arr[mid] == target) return mid;
        if (arr[mid] < target) return binarySearch(arr, target, mid + 1, high);
        return binarySearch(arr, target, low, mid - 1);
    }

    public static void main(String[] args) {
        // Fibonacci
        System.out.println("Fibonacci:");
        for (int i = 0; i <= 10; i++) {
            System.out.println("fib(" + i + ") = " + fib(i));
        }

        // Factorial
        System.out.println("Factorial:");
        System.out.println("5! = " + factorial(5));
        System.out.println("10! = " + factorial(10));
        System.out.println("20! = " + factorial(20));

        // Array sum
        int[] nums = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
        System.out.println("Sum of 1..10 = " + sumArray(nums, 0));

        // Binary search
        int[] sorted = {2, 5, 8, 12, 16, 23, 38, 56, 72, 91};
        int target = 23;
        int pos = binarySearch(sorted, target, 0, sorted.length - 1);
        System.out.println(target + " found at index: " + pos);

        int notFound = 50;
        pos = binarySearch(sorted, notFound, 0, sorted.length - 1);
        System.out.println(notFound + " found at index: " + pos);
    }
}
