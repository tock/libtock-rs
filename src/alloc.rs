use crate::drivers;
use crate::leds::LedsDriver;
use crate::result::TockResult;
use crate::timer::Duration;
use crate::timer::ParallelSleepDriver;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::executor;
use core::ptr;
use core::ptr::NonNull;
use futures::future;
use linked_list_allocator::Heap;

pub static mut HEAP: Heap = Heap::empty();

struct TockAllocator;

unsafe impl GlobalAlloc for TockAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HEAP.allocate_first_fit(layout)
            .ok()
            .map_or(ptr::null_mut(), NonNull::as_ptr)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP.deallocate(NonNull::new_unchecked(ptr), layout)
    }
}

#[global_allocator]
static ALLOCATOR: TockAllocator = TockAllocator;

#[alloc_error_handler]
unsafe fn alloc_error_handler(_: Layout) -> ! {
    executor::block_on(async {
        let mut drivers = drivers::retrieve_drivers_unsafe();

        let leds_driver = drivers.leds.init_driver();
        let mut timer_driver = drivers.timer.create_timer_driver();
        let timer_driver = timer_driver.activate();

        if let (Ok(leds_driver), Ok(timer_driver)) = (leds_driver, timer_driver) {
            let _ = cycle_all_leds(&leds_driver, &timer_driver).await;
        } else {
            future::pending::<()>().await
        }
        loop {}
    })
}

async fn cycle_all_leds(
    leds_driver: &LedsDriver<'_>,
    timer_driver: &ParallelSleepDriver<'_>,
) -> TockResult<()> {
    loop {
        for led in leds_driver.leds() {
            led.on()?;
            timer_driver.sleep(Duration::from_ms(100)).await?;
            led.off()?;
        }
    }
}
