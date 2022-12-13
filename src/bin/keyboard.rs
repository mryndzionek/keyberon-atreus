#![deny(warnings)]
#![no_main]
#![no_std]

#[rtic::app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [PVD])]
mod app {

    use keyberon::action::{d, k, l, m, Action, Action::*, HoldTapAction, HoldTapConfig};
    use keyberon::debounce::Debouncer;
    use keyberon::key_code::KbHidReport;
    use keyberon::key_code::KeyCode::*;
    use keyberon::layout::{CustomEvent, Event, Layout};
    use keyberon::matrix::Matrix;
    use keyberon_atreus as _;

    use stm32f1xx_hal::gpio::{gpioc::PC13, EPin, Input, Output, PullUp, PushPull};
    use stm32f1xx_hal::prelude::*;
    use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
    use stm32f1xx_hal::{pac, timer};
    use usb_device::bus::UsbBusAllocator;
    use usb_device::class::UsbClass as _;
    use usb_device::device::UsbDeviceState;
    use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};

    type UsbClass = keyberon::Class<'static, UsbBusType, ()>;
    type UsbDevice = usb_device::device::UsbDevice<'static, UsbBusType>;

    static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;

    const VID: u16 = 0x16c0;
    const PID: u16 = 0x27db;

    const LCTL_ESC: Action<()> = HoldTap(&HoldTapAction {
        timeout: 200,
        tap_hold_interval: 0,
        config: HoldTapConfig::Default,
        hold: k(LCtrl),
        tap: k(Escape),
    });

    const RALT_EDIT: Action<()> = HoldTap(&HoldTapAction {
        timeout: 200,
        tap_hold_interval: 0,
        config: HoldTapConfig::Default,
        hold: k(RAlt),
        tap: d(4),
    });

    const TILD: Action<()> = m(&[LShift, Grave].as_slice());
    const EXLM: Action<()> = m(&[LShift, Kb1].as_slice());
    const AT: Action<()> = m(&[LShift, Kb2].as_slice());
    const HASH: Action<()> = m(&[LShift, Kb3].as_slice());
    const DLR: Action<()> = m(&[LShift, Kb4].as_slice());
    const PERC: Action<()> = m(&[LShift, Kb5].as_slice());
    const CIRC: Action<()> = m(&[LShift, Kb6].as_slice());
    const AMPR: Action<()> = m(&[LShift, Kb7].as_slice());
    const ASTR: Action<()> = m(&[LShift, Kb8].as_slice());
    const LPRN: Action<()> = m(&[LShift, Kb9].as_slice());
    const RPRN: Action<()> = m(&[LShift, Kb0].as_slice());
    const UNDS: Action<()> = m(&[LShift, Minus].as_slice());
    const PLUS: Action<()> = m(&[LShift, Equal].as_slice());
    const LCBR: Action<()> = m(&[LShift, LBracket].as_slice());
    const RCBR: Action<()> = m(&[LShift, RBracket].as_slice());
    const PIPE: Action<()> = m(&[LShift, Bslash].as_slice());
    const COPY: Action<()> = m(&[LCtrl, C].as_slice());
    const PASTE: Action<()> = m(&[LCtrl, V].as_slice());
    const VSFMT: Action<()> = m(&[LCtrl, K, F].as_slice());

    #[rustfmt::skip]
    pub const LAYERS: keyberon::layout::Layers<14, 4, 5, ()> = [
        [
            [k(Tab),    k(Q),     k(W),    k(E),    k(R), k(T),     Trans,     Trans,     k(Y),      k(U), k(I),     k(O),    k(P),      k(Minus)],
            [LCTL_ESC,  k(A),     k(S),    k(D),    k(F), k(G),     Trans,     Trans,     k(H),      k(J), k(K),     k(L),    k(SColon), k(Quote)],
            [k(LShift), k(Z),     k(X),    k(C),    k(V), k(B),     l(3),      k(RShift), k(N),      k(M), k(Comma), k(Dot),  k(Slash),  k(Enter)],
            [k(Grave),  k(LCtrl), k(LAlt), k(LGui), l(1), k(Space), RALT_EDIT, k(RAlt),   k(BSpace), l(2), k(Left),  k(Down), k(Up),     k(Right)],
        ],
        [
            [TILD,      EXLM,  AT,    HASH,  DLR,    PERC,   Trans, Trans, CIRC,   AMPR,   ASTR,             LPRN,            RPRN,          k(Delete)],
            [k(Delete), k(F1), k(F2), k(F3), k(F4),  k(F5),  Trans, Trans, k(F6),  UNDS,   PLUS,             LCBR,            RCBR,          PIPE],
            [Trans,     k(F7), k(F8), k(F9), k(F10), k(F11), Trans, Trans, k(F12), k(End), Trans,            Trans,           Trans,         Trans],
            [Trans,     Trans, Trans, Trans, Trans,  Trans,  Trans, Trans, Trans,  Trans,  k(MediaNextSong), k(MediaVolDown), k(MediaVolUp), k(MediaPlayPause)],
        ],
        [
            [k(Grave),  k(Kb1), k(Kb2), k(Kb3), k(Kb4), k(Kb5), Trans, Trans, k(Kb6), k(Kb7),   k(Kb8),           k(Kb9),          k(Kb0),        k(Delete)],
            [k(Delete), k(F1),  k(F2),  k(F3),  k(F4),  k(F5),  Trans, Trans, k(F6),  k(Minus), k(Equal),         k(LBracket),     k(RBracket),   k(Bslash)],
            [Trans,     k(F7),  k(F8),  k(F9),  k(F10), k(F11), Trans, Trans, k(F12), k(End),   Trans,            Trans,           Trans,         Trans],
            [Trans,     Trans,  Trans,  Trans,  Trans,  Trans,  Trans, Trans, Trans,  Trans,    k(MediaNextSong), k(MediaVolDown), k(MediaVolUp), k(MediaPlayPause)],
        ],
        [
            [TILD,      EXLM,  AT,    HASH,  DLR,    PERC,   Trans, Trans, CIRC,       AMPR,    k(Up),            LPRN,           RPRN,           k(Delete)],
            [k(Delete), k(F1), k(F2), k(F3), k(F4),  k(F5),  Trans, Trans, k(F6),      k(Left), k(Down),          k(Right),        RCBR,          PIPE],
            [Trans,     k(F7), k(F8), k(F9), k(F10), k(F11), Trans, Trans, k(F12),     k(End),  Trans,            Trans,           Trans,         Trans],
            [Trans,     Trans, Trans, Trans, Trans,  Trans,  Trans, Trans, k(PgDown),  k(PgUp), k(MediaNextSong), k(MediaVolDown), k(MediaVolUp), k(MediaPlayPause)],
        ],
        [
            [k(Tab),    k(Q),     k(W),    k(E),    k(R),  k(T),     Trans, Trans,     k(Y),      k(U), k(I),     k(O),    k(P),      k(Minus)],
            [LCTL_ESC,  k(A),     k(S),    PASTE,   COPY,  k(G),     Trans, Trans,     k(H),      k(J), k(K),     k(L),    k(SColon), k(Quote)],
            [k(LShift), k(Z),     k(X),    k(C),    VSFMT, k(B),     l(3),  k(RShift), k(N),      k(M), k(Comma), k(Dot),  k(Slash),  k(Enter)],
            [k(Grave),  k(LCtrl), k(LAlt), k(LGui), l(1),  k(Space), d(0),  k(RAlt),   k(BSpace), l(2), k(Left),  k(Down), k(Up),     k(Right)],
        ],
    ];

    #[shared]
    struct Shared {
        usb_dev: UsbDevice,
        usb_class: UsbClass,
        #[lock_free]
        layout: Layout<14, 4, 5, ()>,
    }

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        matrix: Matrix<EPin<Input<PullUp>>, EPin<Output<PushPull>>, 14, 4>,
        debouncer: Debouncer<[[bool; 14]; 4]>,
        timer: timer::CountDownTimer<pac::TIM3>,
    }

    #[init]
    fn init(mut c: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("Starting Keyberon");

        // workaround, see: https://github.com/knurling-rs/defmt/issues/322
        #[cfg(debug_assertions)]
        c.device.DBGMCU.cr.modify(|_, w| {
            w.dbg_sleep().set_bit();
            w.dbg_standby().set_bit();
            w.dbg_stop().set_bit()
        });
        #[cfg(debug_assertions)]
        c.device.RCC.ahbenr.modify(|_, w| w.dma1en().enabled());

        let mut flash = c.device.FLASH.constrain();
        let rcc = c.device.RCC.constrain();

        // set 0x424C in DR10 for dfu on reset
        let bkp = rcc.bkp.constrain(c.device.BKP, &mut c.device.PWR);
        bkp.write_data_register_low(9, 0x424C);

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);

        let mut gpioa = c.device.GPIOA.split();
        let mut gpiob = c.device.GPIOB.split();
        let mut gpioc = c.device.GPIOC.split();

        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low();
        cortex_m::asm::delay(clocks.sysclk().0 / 100);

        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        led.set_high();

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

        let usb = Peripheral {
            usb: c.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        let usb_bus = unsafe {
            USB_BUS = Some(UsbBus::new(usb));
            USB_BUS.as_ref().unwrap()
        };

        let tel = concat!("When found call: ", include_str!("../../tel.txt"));
        let usb_class = keyberon::new_class(usb_bus, ());
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(VID, PID))
            .manufacturer(&tel[0..tel.len() - 1])
            .product("Atreus_52")
            .serial_number(env!("CARGO_PKG_VERSION"))
            .build();

        let mut timer = timer::Timer::tim3(c.device.TIM3, &clocks).start_count_down(1.khz());
        timer.listen(timer::Event::Update);

        let mut afio = c.device.AFIO.constrain();
        let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

        let matrix = Matrix::new(
            [
                gpioa.pa0.into_pull_up_input(&mut gpioa.crl).erase(),
                gpioa.pa1.into_pull_up_input(&mut gpioa.crl).erase(),
                gpioa.pa2.into_pull_up_input(&mut gpioa.crl).erase(),
                gpioa.pa3.into_pull_up_input(&mut gpioa.crl).erase(),
                gpioa.pa4.into_pull_up_input(&mut gpioa.crl).erase(),
                gpioa.pa5.into_pull_up_input(&mut gpioa.crl).erase(),
                gpiob.pb6.into_pull_up_input(&mut gpiob.crl).erase(),
                gpiob.pb7.into_pull_up_input(&mut gpiob.crl).erase(),
                gpioa.pa8.into_pull_up_input(&mut gpioa.crh).erase(),
                gpioa.pa10.into_pull_up_input(&mut gpioa.crh).erase(),
                gpioa.pa9.into_pull_up_input(&mut gpioa.crh).erase(),
                pa15.into_pull_up_input(&mut gpioa.crh).erase(),
                pb3.into_pull_up_input(&mut gpiob.crl).erase(),
                pb4.into_pull_up_input(&mut gpiob.crl).erase(),
            ],
            [
                gpiob.pb0.into_push_pull_output(&mut gpiob.crl).erase(),
                gpiob.pb10.into_push_pull_output(&mut gpiob.crh).erase(),
                gpiob.pb5.into_push_pull_output(&mut gpiob.crl).erase(),
                gpiob.pb8.into_push_pull_output(&mut gpiob.crh).erase(),
            ],
        );

        (
            Shared {
                usb_dev,
                usb_class,
                layout: Layout::new(&LAYERS),
            },
            Local {
                led,
                timer,
                debouncer: Debouncer::new([[false; 14]; 4], [[false; 14]; 4], 5),
                matrix: matrix.unwrap(),
            },
            init::Monotonics(),
        )
    }

    #[idle(local = [])]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(binds = USB_HP_CAN_TX, priority = 3, shared = [usb_dev, usb_class])]
    fn usb_tx(c: usb_tx::Context) {
        let r1 = c.shared.usb_dev;
        let r2 = c.shared.usb_class;
        (r1, r2).lock(|dev, class| {
            usb_poll(dev, class);
        });
    }

    #[task(binds = USB_LP_CAN_RX0, priority = 3, shared = [usb_dev, usb_class])]
    fn usb_rx(c: usb_rx::Context) {
        let r1 = c.shared.usb_dev;
        let r2 = c.shared.usb_class;
        (r1, r2).lock(|dev, class| {
            usb_poll(dev, class);
        });
    }

    #[task(priority = 2, capacity = 8, shared = [layout])]
    fn handle_event(c: handle_event::Context, event: Event) {
        c.shared.layout.event(event)
    }

    #[task(priority = 2, local = [led], shared = [usb_dev, usb_class, layout])]
    fn tick_keyberon(mut c: tick_keyberon::Context) {
        if c.shared.layout.current_layer() == 4 {
            c.local.led.set_low()
        } else {
            c.local.led.set_high()
        };
        let tick = c.shared.layout.tick();
        if c.shared.usb_dev.lock(|d| d.state()) != UsbDeviceState::Configured {
            return;
        }
        match tick {
            CustomEvent::Release(()) => unsafe { cortex_m::asm::bootload(0x1FFFC800 as _) },
            _ => (),
        }
        let report: KbHidReport = c.shared.layout.keycodes().collect();
        if !c
            .shared
            .usb_class
            .lock(|k| k.device_mut().set_keyboard_report(report.clone()))
        {
            return;
        }
        while let Ok(0) = c.shared.usb_class.lock(|k| k.write(report.as_bytes())) {}
    }

    #[task(binds = TIM3, priority = 1, local = [matrix, debouncer, timer])]
    fn tick(c: tick::Context) {
        c.local.timer.clear_update_interrupt_flag();

        for event in c.local.debouncer.events(c.local.matrix.get().unwrap()) {
            handle_event::spawn(event).unwrap();
        }
        tick_keyberon::spawn().unwrap();
    }

    fn usb_poll(usb_dev: &mut UsbDevice, keyboard: &mut UsbClass) {
        if usb_dev.poll(&mut [keyboard]) {
            keyboard.poll();
        }
    }
}
