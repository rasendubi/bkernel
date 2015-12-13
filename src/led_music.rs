use led;

pub fn led_fun(tt:u32) {
    let mut a:u32 = 10;
    delay(tt);
    led::LD3.turn_off();
    led::LD4.turn_off();
    led::LD5.turn_off();
    led::LD6.turn_off();
    delay(tt);
    led::LD3.turn_on();
    led::LD4.turn_on();
    led::LD5.turn_on();
    led::LD6.turn_on();
    delay(tt);
    while a>0 {
        play_led_step(tt);
        a = a-1;
    }
    delay(tt);
    led::LD3.turn_on();
    led::LD4.turn_on();
    led::LD5.turn_on();
    led::LD6.turn_on();
}

fn delay(a:u32) {
    unsafe {
        let i: ::stm32f4::volatile::RW<u32> = ::core::mem::uninitialized();
        i.set(a);
        while i.get()>0 {
            i.update(|x| x - 1);
        }
    }
}

fn play_led_step(tt:u32) {
    led::LD3.turn_on();
    delay(tt);
    led::LD3.turn_off();

    delay(tt/10);

    led::LD4.turn_on();
    delay(tt);
    led::LD4.turn_off();

    delay(tt/10);
    
    led::LD5.turn_on();
    delay(tt);
    led::LD5.turn_off();

    delay(tt/10);
    
    led::LD6.turn_on();
    delay(1000);
    led::LD6.turn_off();

    delay(1000);
}
