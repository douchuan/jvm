import java.lang.*;

// enum showing Mobile prices
public enum EnumMobile {
   Samsung(400), Nokia(250);
  
   int price;
   EnumMobile(int p) {
      price = p;
   }
   int showPrice() {
      return price;
   } 
}
