trait PaymentProcessor {
    fn process(&self, amount: f64) -> Result<String,String>;
}
struct CreditCard;
struct Paypal;

impl PaymentProcessor for CreditCard {
    fn process(&self, amount: f64) -> Result<String, String> {
        Ok(format!("Charged ${} to card", amount))
    }
}

impl PaymentProcessor for Paypal {
    fn process(&self, amount: f64) -> Result<String, String> {
        Ok(format!("Paid ${} via PayPal", amount))
    }
}

fn checkout(processor: &dyn PaymentProcessor, amount: f64){
    match processor.process(amount) {
        Ok(msg) => println!("{}",msg),
        Err(err) => println!("Error: {}", err)
    }
}
fn main(){
    checkout(&CreditCard, 99.99);
    checkout(&Paypal, 49.99);
}