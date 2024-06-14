#![no_std]
#![no_main]

#[rtic::app(device = nrf52840_hal::pac, dispatchers=[SWI0_EGU0])]
mod app {
    const MAX_TIME: i32 = 50;
    const SCREEN_WIDTH: i32 = 240;
    const SCREEN_HEIGHT: i32 = 240;
    const CENTER: i32 = 120;

    use core::fmt::Write;
    use display_interface_spi::SPIInterface;
    use embedded_graphics::{
        mono_font::{ascii::FONT_9X18, MonoTextStyle},
        pixelcolor::Rgb565,
        prelude::*,
        primitives::{PrimitiveStyle, Rectangle, PrimitiveStyleBuilder},
        text::Text,
    };
    use heapless::String;
    use nrf52840_hal::{
        clocks::Clocks,
        gpio::{p0, p0::P0_12, p0::P0_13, p1, p1::P1_03, p1::P1_05, Level, Output, PushPull},
        gpiote::Gpiote,
        pac::SPIM0,
        spim,
        Delay,
    };
    use panic_halt as _;
    use st7789::{Orientation, ST7789};
    use systick_monotonic::{ExtU64, Systick};

    #[monotonic(binds = SysTick, default = true)]
    type Mono = Systick<100>; 
    #[shared]
    struct Shared {
        running: bool,
        time_left: i32,
    }

    #[local]
    struct Local {
        display: ST7789<SPIInterface<spim::Spim<SPIM0>, P0_13<Output<PushPull>>, P0_12<Output<PushPull>>>, P1_03<Output<PushPull>>, P1_05<Output<PushPull>>>,
        gpiote: Gpiote,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let port0 = p0::Parts::new(ctx.device.P0);
        let port1 = p1::Parts::new(ctx.device.P1);
    
        let button_a = port1.p1_02.into_pullup_input().degrade();
        let button_b = port1.p1_10.into_pullup_input().degrade();
        let gpiote = Gpiote::new(ctx.device.GPIOTE);
        gpiote.channel0().input_pin(&button_a).hi_to_lo().enable_interrupt();
        gpiote.channel1().input_pin(&button_b).hi_to_lo().enable_interrupt();
    
        let cs_pin = port0.p0_12.into_push_pull_output(Level::High);
        let dc_pin = port0.p0_13.into_push_pull_output(Level::Low);
        let sck_pin = port0.p0_14.into_push_pull_output(Level::Low).degrade();
        let mosi_pin = port0.p0_15.into_push_pull_output(Level::Low).degrade();
        let rst_pin = port1.p1_03.into_push_pull_output(Level::Low);
        let backlight_pin = port1.p1_05.into_push_pull_output(Level::Low);
    
        let spi = spim::Spim::new(
            ctx.device.SPIM0,
            spim::Pins {
                sck: Some(sck_pin),
                miso: None,
                mosi: Some(mosi_pin),
            },
            spim::Frequency::M8,
            spim::MODE_3,
            122,
        );
    
        let spi_display = SPIInterface::new(spi, dc_pin, cs_pin);
        let mut display = ST7789::new(spi_display, Some(rst_pin), Some(backlight_pin), 240, 240);
        
        // First, create a Delay object to initialize the display
        let mut delay = Delay::new(ctx.core.SYST);
        display.init(&mut delay).unwrap();
        display.set_orientation(Orientation::LandscapeSwapped).unwrap();
        display.clear(Rgb565::BLACK).unwrap();
        
        // Free the SYST from Delay
        let syst = delay.free();
    
        // Now, use the freed SYST for the monotonic timer
        let mono = Systick::new(syst, 64_000_000);
    
        Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
    
        (
            Shared { running: false, time_left: MAX_TIME },
            Local { display, gpiote },
            init::Monotonics(mono),
        )
    }
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            rtic::export::wfi();
        }
    }

    #[task(shared = [running, time_left], local = [display])]
    fn update_display(mut ctx: update_display::Context) {
        let running = ctx.shared.running.lock(|r| *r);
        let time_left = ctx.shared.time_left.lock(|t| *t);

        ctx.local.display.clear(Rgb565::BLACK).unwrap();

        let color = if running && time_left <= 1 {
            Rgb565::RED
        } else if running {
            Rgb565::YELLOW
        } else {
            Rgb565::GREEN
        };

        let progress_height = (time_left as f32 / MAX_TIME as f32 * SCREEN_HEIGHT as f32) as i32;

        
        Rectangle::new(
            Point::new(0, SCREEN_HEIGHT - progress_height),
            Size::new(SCREEN_WIDTH  as u32 + 100 as u32, progress_height as u32),
        )
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(ctx.local.display)
        .unwrap();

        let mut text: String<8> = String::new();
        if !running {
            let mut instructions: String<45> = String::new();
            write!(instructions, "<-- Start Timer\nSet Time-->\nTime: {:02}s", time_left).unwrap();
            Text::new(
                &instructions,
                Point::new(CENTER + 10, CENTER- 30),
                MonoTextStyle::new(&FONT_9X18, Rgb565::WHITE),
            )
            .draw(ctx.local.display)
            .unwrap();
        }else{
            if time_left <= 1 {
                write!(text, "BEEEP").unwrap();
            } else {
                write!(text, "{:02}", time_left).unwrap();
            }
            Text::new(
                &text,
                Point::new(CENTER + 50, CENTER),
                MonoTextStyle::new(&FONT_9X18, Rgb565::WHITE),
            )
            .draw(ctx.local.display)
            .unwrap();

            if time_left <= 0 {
                ctx.shared.running.lock(|r| *r = false);
            } else {
                ctx.shared.time_left.lock(|t| *t -= 1);
                update_display::spawn_after(1.secs()).unwrap();
            }
        }
    }
    #[task(binds = GPIOTE, local = [gpiote], shared = [running, time_left])]
    fn handle_buttons(mut ctx: handle_buttons::Context) {
        if ctx.local.gpiote.channel0().is_event_triggered() {
            ctx.local.gpiote.channel0().reset_events();
            ctx.shared.running.lock(|r| *r = true);
            update_display::spawn().unwrap();
        } else if ctx.local.gpiote.channel1().is_event_triggered() {
            ctx.local.gpiote.channel1().reset_events();
            ctx.shared.running.lock(|r| *r = false);
            ctx.shared.time_left.lock(|t| *t = (*t + 5) % (MAX_TIME));
            update_display::spawn().unwrap();
        }
    }

}