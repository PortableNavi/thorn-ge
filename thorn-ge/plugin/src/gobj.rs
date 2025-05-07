use thorn::prelude::*;


pub struct GobjA;
impl Gobject for GobjA
{
    fn test_event_a(&mut self)
    {
        println!("I am A !!");
    }

    fn reset(&mut self)
    {
        println!("A was initialized");
    }

    fn destroy(&mut self)
    {
        println!("A was destroyed");
    }
}


pub struct GobjB;
impl Gobject for GobjB
{
    fn test_event_a(&mut self)
    {
        println!("I am B !!");
    }


    fn reset(&mut self)
    {
        println!("B was initialized");
    }

    fn destroy(&mut self)
    {
        println!("B was destroyed");
    }
}
