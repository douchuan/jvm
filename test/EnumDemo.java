import java.lang.*;

// enum showing Mobile prices
enum Mobile {
   Samsung(400), Nokia(250);
  
   int price;
   Mobile(int p) {
      price = p;
   }
   int showPrice() {
      return price;
   } 
}

public class EnumDemo {

   public static void main(String args[]) {

      System.out.println("Enums can never be cloned...");
      EnumDemo t = new EnumDemo() {
         protected final Object clone() throws CloneNotSupportedException {
            throw new CloneNotSupportedException();
         }
      }; 

      System.out.println("CellPhone List:");
      System.out.println("CellPhone List: " + Mobile.Samsung);
      System.out.println("CellPhone List: " + Mobile.values());
      for(Mobile m : Mobile.values()) {
         System.out.println(m + " costs " + m.showPrice() + " dollars");
      }
   }
}
