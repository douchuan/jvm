public class OopEncapsulation {
    private int balance;
    private String owner;

    public OopEncapsulation(String owner, int balance) {
        this.owner = owner;
        this.balance = balance;
    }

    public String getOwner() {
        return owner;
    }

    public int getBalance() {
        return balance;
    }

    public void deposit(int amount) {
        if (amount > 0) {
            balance += amount;
        }
    }

    public boolean withdraw(int amount) {
        if (amount > 0 && amount <= balance) {
            balance -= amount;
            return true;
        }
        return false;
    }

    public static void main(String[] args) {
        OopEncapsulation account = new OopEncapsulation("Alice", 1000);
        System.out.println("Owner: " + account.getOwner());
        System.out.println("Balance: " + account.getBalance());

        account.deposit(500);
        System.out.println("After deposit 500: " + account.getBalance());

        account.withdraw(200);
        System.out.println("After withdraw 200: " + account.getBalance());

        account.withdraw(5000);
        System.out.println("After failed withdraw 5000: " + account.getBalance());
    }
}
