
// Base of peripherals and base of GPIO controller.
const BCM2835_PERIPH_BASE: usize = 0x2000_0000;
const BCM2835_GPIO_BASE:   usize   = BCM2835_PERIPH_BASE + 0x20_0000;

// Paging definitions.
const RPI_PAGE_SIZE:  usize   = 4096;
const RPI_BLOCK_SIZE: usize  = 4096;


// Helper macros for accessing GPIO registers.

// enum Pin {

// }

// Utiliser un array statique de 0 à REG gpioMax?
struct Gpio {
    base_addr: *mut usize,
    size: usize,
}

impl Drop for Gpio {
  fn drop(&mut self) {
    unsafe {
      libc::munmap(self.base_addr as *mut libc::c_void, self.size);
    }
  }
}

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
    Err("Impossible to map gpio adress range")
  }
}

impl Gpio {
  pub fn new() -> Result<Self, & 'static str> {
  // Setup the access to memory-mapped I/O.
  let mmap_fd = open_mem()?;

  let base_addr = unsafe { 
    libc::mmap(
    0 as *mut libc::c_void,
    RPI_BLOCK_SIZE,
    libc::PROT_READ | libc::PROT_WRITE,
    libc::MAP_SHARED,
    mmap_fd,
    BCM2835_GPIO_BASE as i64)
  };

  // The file descriptor can be closed after mmaping see Posix doc mmap.
  unsafe { libc::close(mmap_fd); }
  if base_addr == libc::MAP_FAILED {
    return Err("Cannot setup mmapped GPIO.\n");
  }

    Ok(Gpio { base_addr: base_addr as *mut usize, size: RPI_BLOCK_SIZE })
  }

  pub fn as_input(&self, gpio: isize) {
      let p = offset_conf(self.base_addr, gpio);
      unsafe { *p &= input_mask(gpio); }
  }

  pub fn as_output(&self, gpio: isize) {
    self.as_input(gpio);
    let p = offset_conf(self.base_addr, gpio);
    unsafe { *p |= output_mask(gpio); }
  }

  pub fn write(&self, gpio: isize) {
    assert!(gpio < 32, "Pin are between 0 and 32.");
    let p = offset_register(self.base_addr, 0b000_1100, gpio);
    unsafe { *p = 1 << gpio };
  }

  pub fn clear(&self, gpio: isize) {
    assert!(gpio < 32, "Pin are between 0 and 32.");
    let p = offset_register(self.base_addr, 0b0010_1000, gpio);
    unsafe { *p = 1 << gpio }
  }

  pub fn read(&self, gpio: isize) -> usize {
    let p = offset_register(self.base_addr, 0b0011_0100, gpio);
    unsafe { *p >> gpio & 0x1 }
  }
}

fn offset_conf(ptr: *mut usize , gpio: isize) -> *mut usize {
  unsafe { ptr.offset(gpio / 10) }
}

fn output_mask(gpio: isize) -> usize {
  0x1 << ((gpio % 10) * 3)
}
    
fn input_mask(gpio: isize) -> usize {
  !( 0x7 << ( gpio % 10) * 3)
}

fn offset_register(ptr: *mut usize, addr: isize, gpio: isize) -> *mut usize {
  let base_addr = addr / std::mem::size_of::<isize>() as isize;
  unsafe { ptr.offset(base_addr + (gpio / 32)) }
}

fn main() {
  println!("Hello, world!");
  let gpio = Gpio::new();
}
