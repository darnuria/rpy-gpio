
use volatile_register::{RW, RO};

// TODO use function from bcm_host
// bcm_host_get_peripheral_address()

// Base of peripherals and base of GPIO controller.
const BCM2837_PERIPH_BASE: usize = 0x3F00_0000;
const BCM2837_GPIO_BASE:   usize   = BCM2837_PERIPH_BASE + 0x20_0000;

// Paging definitions.
const RPI_PAGE_SIZE:  usize   = 4096;
const RPI_BLOCK_SIZE: usize  = 4096;


// Helper macros for accessing GPIO registers.

// Memory layout of the gpio pins.
#[repr(C)]
pub struct Gpio {
  fsel:   [RW<u32>; 7],
  set:    [RW<u32>; 3],
  clr:    [RW<u32>; 3],
  lev:    [RO<u32>; 3],
  eds:    [RW<u32>; 3],
  ren:    [RW<u32>; 3],
  fen:    [RW<u32>; 3],
  hen:    [RW<u32>; 3],
  len:    [RW<u32>; 3],
  aren:   [RW<u32>; 3],
  afen:   [RW<u32>; 3],
  pud:    [RW<u32>; 1],
  pudclk: [RW<u32>; 3],
  test:   [RW<u32>; 1],
}

// impl Drop for Gpio {
//   fn drop(&mut self) {
//     println!("Hi i'am droping!");
//     for i in 0..32 {
//       self.write(i, false);
//     }
//     unsafe {
//       libc::munmap((self as * mut Self) as (* mut libc::c_void), RPI_BLOCK_SIZE);
//     }
//   }
// }

// TODO rewrite with openOption and OpenOptionExt
fn open_mem() -> Result<libc::c_int, &'static str> {
  if let Ok(filename) = std::ffi::CString::new("/dev/mem") {
    let mmap_fd = unsafe { 
      libc::open(
        filename.as_ptr(),
        libc::O_RDWR | libc::O_SYNC
      )
    };
    if mmap_fd < 0 {
      Err("Cannot open /dev/mem")
    } else {
      Ok(mmap_fd)
    }
  } else {
    Err("Cannot create CString")
  }
}

// Improve with https://rust-embedded.github.io/book/static-guarantees/design-contracts.html
impl Gpio {
  pub fn new<'a>() -> Result<&'a mut Self, & 'static str> {
  // Setup the access to memory-mapped I/O.
  let mmap_fd = open_mem()?;

  let base_addr = unsafe { 
    libc::mmap(
      0 as *mut libc::c_void,
      RPI_BLOCK_SIZE,
      libc::PROT_READ | libc::PROT_WRITE,
      libc::MAP_SHARED,
      mmap_fd,
      BCM2837_GPIO_BASE as i32
    )
  };

  // The file descriptor can be closed after mmaping see Posix doc mmap.
  unsafe { libc::close(mmap_fd); }
  if base_addr == libc::MAP_FAILED {
    return Err("Cannot setup mmapped GPIO.\n");
  }

    // Rust magic to convert pointer to static ref.
    Ok( unsafe { &mut *(base_addr as *mut Gpio) })
  }

  pub fn as_input(&self, pin: usize) -> &Self {
    let reg = pin / 10;
    let bit = (pin % 10) * 3;
    let mask = 0b111 << bit;
    let val = self.fsel[reg].read() & !mask;
    unsafe { self.fsel[reg].write(val); }
    self
  }

  pub fn as_output(&self, pin: usize) -> &Self{
    let reg = pin / 10;
    let bit = (pin % 10) * 3;
    let mask = 0b111 << bit;
    let val = (self.fsel[reg].read() & !mask) | ((1 << bit) & mask);
    unsafe { self.fsel[reg].write(val); }
    self
  }

  pub fn write(&self, pin: usize, val: bool) {
    assert!(pin < 32, "There is less than 32 pin on the raspberryPi");
    if val {
      unsafe { self.set[pin / 32].write(1 << pin); }
    } else {
      unsafe { self.clr[pin / 32].write(1 << pin); }
    }
  }

  pub fn read(&self, pin: usize) -> u32 {
    self.lev[pin].read() >> pin & 0x1
  }
}

fn main() {
  println!("Hello gpio");
  let gpio = Gpio::new().unwrap();
  gpio.as_output(3).as_output(4);
  let mut val = true;
  let ten_millis = std::time::Duration::from_millis(100);
  for _ in 0..16 {
    val = !val;
    gpio.write(3, !val);
    gpio.write(4, val);
    std::thread::sleep(ten_millis);
  }

  // Closing todo: Move inside drop need refatoring.
  for i in 0..32 {
      gpio.write(i, false);
  }
  unsafe {
    libc::munmap((gpio as * mut Gpio) as (* mut libc::c_void), RPI_BLOCK_SIZE);
  }
}
