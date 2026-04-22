interface IRun { void run(); }
public class Interfaces implements IRun {
    public static int constant = 42;

    public void run() {
        System.out.println("Running!");
    }

    public static void main(String[] args) {
        Interfaces impl = new Interfaces();
        impl.run();
        System.out.println("constant = " + Interfaces.constant);
        System.out.println("Interfaces test OK");
    }
}
