public class GCDemo {
    // Phase 2: Requires GC to be implemented
    // Without GC, this will eventually cause OOM.
    // Once GC is implemented, this should run without issues.

    static class Node {
        int value;
        Node next;

        Node(int value) {
            this.value = value;
        }
    }

    public static void main(String[] args) {
        // Allocate many objects
        int total = 10000;
        long count = 0;

        System.out.println("Allocating objects...");

        try {
            for (int i = 0; i < total; i++) {
                // Create a linked list segment
                Node head = new Node(0);
                Node current = head;
                for (int j = 1; j < 100; j++) {
                    current.next = new Node(j);
                    current = current.next;
                }
                count += 100;

                if (i % 1000 == 0) {
                    System.out.println("Allocated " + count + " objects so far");
                }
            }
            System.out.println("Total objects allocated: " + count);
            System.out.println("GC Demo completed successfully");
        } catch (OutOfMemoryError e) {
            System.out.println("OutOfMemoryError - GC not yet implemented");
            System.out.println("This test requires Phase 2 (GC) to be completed");
        }
    }
}
