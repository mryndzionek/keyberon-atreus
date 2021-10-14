#![no_main]
#![no_std]

use keyberon_atreus as _;

use keyberon::action::{d, k, l, m, Action, Action::*, HoldTapConfig};
use keyberon::debounce::Debouncer;
use keyberon::key_code::KeyCode::*;
use keyberon::key_code::{KbHidReport, KeyCode};
use keyberon::layout::Layout;
use keyberon::matrix::{Matrix, PressedKeys};

use rtic::app;
use stm32f1xx_hal::gpio::{gpioc::PC13, EPin, Input, Output, PullUp, PushPull};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use stm32f1xx_hal::{pac, timer};
use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass as _;

type UsbClass = keyberon::Class<'static, UsbBusType, ()>;
type UsbDevice = usb_device::device::UsbDevice<'static, UsbBusType>;

const LCTL_ESC: Action<()> = HoldTap {
    timeout: 200,
    tap_hold_interval: 0,
    config: HoldTapConfig::Default,
    hold: &k(LCtrl),
    tap: &k(Escape),
};

const RALT_EDIT: Action<()> = HoldTap {
    timeout: 200,
    tap_hold_interval: 0,
    config: HoldTapConfig::Default,
    hold: &k(RAlt),
    tap: &d(4),
};

const TILD: Action<()> = m(&[LShift, Grave]);
const EXLM: Action<()> = m(&[LShift, Kb1]);
const AT: Action<()> = m(&[LShift, Kb2]);
const HASH: Action<()> = m(&[LShift, Kb3]);
const DLR: Action<()> = m(&[LShift, Kb4]);
const PERC: Action<()> = m(&[LShift, Kb5]);
const CIRC: Action<()> = m(&[LShift, Kb6]);
const AMPR: Action<()> = m(&[LShift, Kb7]);
const ASTR: Action<()> = m(&[LShift, Kb8]);
const LPRN: Action<()> = m(&[LShift, Kb9]);
const RPRN: Action<()> = m(&[LShift, Kb0]);
const UNDS: Action<()> = m(&[LShift, Minus]);
const PLUS: Action<()> = m(&[LShift, Equal]);
const LCBR: Action<()> = m(&[LShift, LBracket]);
const RCBR: Action<()> = m(&[LShift, RBracket]);
const PIPE: Action<()> = m(&[LShift, Bslash]);

#[rustfmt::skip]
pub const LAYERS: keyberon::layout::Layers<()> = &[
    &[
        &[k(Tab),    k(Q),     k(W),    k(E),    k(R), k(T),     Trans,     Trans,     k(Y),      k(U), k(I),     k(O),    k(P),      k(Minus)],
        &[LCTL_ESC,  k(A),     k(S),    k(D),    k(F), k(G),     Trans,     Trans,     k(H),      k(J), k(K),     k(L),    k(SColon), k(Quote)],
        &[k(LShift), k(Z),     k(X),    k(C),    k(V), k(B),     l(3),      k(RShift), k(N),      k(M), k(Comma), k(Dot),  k(Slash),  k(Enter)],
        &[k(Grave),  k(LCtrl), k(LAlt), k(LGui), l(1), k(Space), RALT_EDIT, k(RAlt),   k(BSpace), l(2), k(Left),  k(Down), k(Up),     k(Right)],
    ],
    &[
        &[TILD,      EXLM,  AT,    HASH,  DLR,    PERC,   Trans, Trans, CIRC,   AMPR,   ASTR,             LPRN,            RPRN,          k(Delete)],
        &[k(Delete), k(F1), k(F2), k(F3), k(F4),  k(F5),  Trans, Trans, k(F6),  UNDS,   PLUS,             LCBR,            RCBR,          PIPE],
        &[Trans,     k(F7), k(F8), k(F9), k(F10), k(F11), Trans, Trans, k(F12), k(End), Trans,            Trans,           Trans,         Trans],
        &[Trans,     Trans, Trans, Trans, Trans,  Trans,  Trans, Trans, Trans,  Trans,  k(MediaNextSong), k(MediaVolDown), k(MediaVolUp), k(MediaPlayPause)],
    ],
    &[
        &[k(Grave),  k(Kb1), k(Kb2), k(Kb3), k(Kb4), k(Kb5), Trans, Trans, k(Kb6), k(Kb7),   k(Kb8),           k(Kb9),          k(Kb0),        k(Delete)],
        &[k(Delete), k(F1),  k(F2),  k(F3),  k(F4),  k(F5),  Trans, Trans, k(F6),  k(Minus), k(Equal),         k(LBracket),     k(RBracket),   k(Bslash)],
        &[Trans,     k(F7),  k(F8),  k(F9),  k(F10), k(F11), Trans, Trans, k(F12), k(End),   Trans,            Trans,           Trans,         Trans],
        &[Trans,     Trans,  Trans,  Trans,  Trans,  Trans,  Trans, Trans, Trans,  Trans,    k(MediaNextSong), k(MediaVolDown), k(MediaVolUp), k(MediaPlayPause)],
    ],
    &[
        &[TILD,      EXLM,  AT,    HASH,  DLR,    PERC,   Trans, Trans, CIRC,       AMPR,    k(Up),            LPRN,           RPRN,           k(Delete)],
        &[k(Delete), k(F1), k(F2), k(F3), k(F4),  k(F5),  Trans, Trans, k(F6),      k(Left), k(Down),          k(Right),        RCBR,          PIPE],
        &[Trans,     k(F7), k(F8), k(F9), k(F10), k(F11), Trans, Trans, k(F12),     k(End),  Trans,            Trans,           Trans,         Trans],
        &[Trans,     Trans, Trans, Trans, Trans,  Trans,  Trans, Trans, k(PgDown),  k(PgUp), k(MediaNextSong), k(MediaVolDown), k(MediaVolUp), k(MediaPlayPause)],
    ],
    &[
        &[k(LCtrl), k(RShift), k(LAlt), Trans,          Trans,          Trans,                Trans,                  Trans, Trans, Trans, Trans, Trans, Trans, Trans],
        &[LCTL_ESC, Trans,     Trans,   m(&[LCtrl, D]), k(S),           k(N),                 Trans,                  Trans, Trans, Trans, Trans, Trans, Trans, Trans],
        &[Trans,    Trans,     Trans,   k(Delete),      m(&[LCtrl, G]), Trans,                m(&[LShift, LCtrl, Z]), Trans, Trans, Trans, Trans, Trans, Trans, Trans],
        &[Trans,    Trans,     Trans,   Trans,          Trans,          m(&[LCtrl, Z]),       d(0),                   Trans, Trans, Trans, Trans, Trans, Trans, Trans],
    ],
];

#[app(device = stm32f1xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice,
        usb_class: UsbClass,
        led: PC13<Output<PushPull>>,
        matrix: Matrix<EPin<Input<PullUp>>, EPin<Output<PushPull>>, 14, 4>,
        debouncer: Debouncer<PressedKeys<14, 4>>,
        layout: Layout<()>,
        timer: timer::CountDownTimer<pac::TIM3>,
    }

    #[init]
    fn init(mut c: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
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

        *USB_BUS = Some(UsbBus::new(usb));
        let usb_bus = USB_BUS.as_ref().unwrap();

        let usb_class = keyberon::new_class(usb_bus, ());
        let usb_dev = keyberon::new_device(usb_bus);

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

        init::LateResources {
            usb_dev,
            usb_class,
            led,
            timer,
            debouncer: Debouncer::new(PressedKeys::default(), PressedKeys::default(), 5),
            matrix: matrix.unwrap(),
            layout: Layout::new(LAYERS),
        }
    }

    #[idle(resources = [led])]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            _cx.resources.led.set_high();
            cortex_m::asm::wfi();
            _cx.resources.led.set_low();
        }
    }

    #[task(binds = USB_HP_CAN_TX, priority = 2, resources = [usb_dev, usb_class])]
    fn usb_tx(mut c: usb_tx::Context) {
        usb_poll(&mut c.resources.usb_dev, &mut c.resources.usb_class);
    }

    #[task(binds = USB_LP_CAN_RX0, priority = 2, resources = [usb_dev, usb_class])]
    fn usb_rx(mut c: usb_rx::Context) {
        usb_poll(&mut c.resources.usb_dev, &mut c.resources.usb_class);
    }

    #[task(binds = TIM3, priority = 1, resources = [usb_class, matrix, debouncer, layout, timer])]
    fn tick(mut c: tick::Context) {
        c.resources.timer.clear_update_interrupt_flag();

        for event in c
            .resources
            .debouncer
            .events(c.resources.matrix.get().unwrap())
        {
            c.resources.layout.event(event);
        }
        match c.resources.layout.tick() {
            keyberon::layout::CustomEvent::Release(()) => cortex_m::peripheral::SCB::sys_reset(),
            _ => (),
        }
        send_report(c.resources.layout.keycodes(), &mut c.resources.usb_class);
    }
};

fn send_report(iter: impl Iterator<Item = KeyCode>, usb_class: &mut resources::usb_class<'_>) {
    use rtic::Mutex;
    let report: KbHidReport = iter.collect();
    if usb_class.lock(|k| k.device_mut().set_keyboard_report(report.clone())) {
        while let Ok(0) = usb_class.lock(|k| k.write(report.as_bytes())) {}
    }
}

fn usb_poll(usb_dev: &mut UsbDevice, keyboard: &mut UsbClass) {
    if usb_dev.poll(&mut [keyboard]) {
        keyboard.poll();
    }
}
