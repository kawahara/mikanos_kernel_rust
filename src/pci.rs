use arrayvec::ArrayVec;
use core::fmt::Formatter;
// To use set_bits and set_bit for numbers
use bit_field::BitField;
use x86_64::instructions::port::Port;

// config_address is 32 bit register
struct ConfigAddress(u32);

impl ConfigAddress {
    fn new(bus: u8, device: u8, function: u8, reg_addr: u8) -> Self {
        assert_eq!(reg_addr & 0x3, 0);

        // 00000000 00000000 0000000000 00000000
        let mut value = 0u32;

        // 0-7: register offset (0-255)
        value.set_bits(0..8, u32::from(reg_addr));

        // 8-10: function number (0-7)
        value.set_bits(8..11, u32::from(function));

        // 11-15: device number (0-31)
        value.set_bits(11..16, u32::from(device));

        // 16-23: bus number (16-23)
        value.set_bits(16..24, u32::from(bus));

        // 24-30: reserved (0)
        value.set_bits(24..31, 0);

        // 31: Enable bit. If it set 1, CONFIG_DATA is transferred to PCI configuration spage.
        value.set_bit(31, true);

        Self(value)
    }
}

// PCI configuration port set
#[derive(Debug)]
struct PortSet {
    addr: Port<u32>,
    data: Port<u32>,
}

#[derive(Debug)]
struct Config(spin::Mutex<PortSet>);

static CONFIG: Config = Config(spin::Mutex::new(PortSet {
    addr: Port::new(0x0cf8),
    data: Port::new(0xcfc),
}));

#[derive(Debug, Clone)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub class_code: u32,
    pub header_type: u8,
}

impl core::fmt::Display for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02x}.{:02x}.{:02x} vend={:04x}, class={}, head={:02x}",
            self.bus, self.device, self.function, self.vendor_id, self.class_code, self.header_type
        )
    }
}

pub type Devices = ArrayVec<Device, 32>;

fn read_data(addr: ConfigAddress) -> u32 {
    let mut ports = CONFIG.0.lock();
    unsafe {
        ports.addr.write(addr.0);
        ports.data.read()
    }
}

fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    let addr = ConfigAddress::new(bus, device, function, 0x00);
    (read_data(addr) & 0xffff) as u16
}

fn read_class_code(bus: u8, device: u8, function: u8) -> u32 {
    let addr = ConfigAddress::new(bus, device, function, 0x08);
    read_data(addr)
}

fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    let addr = ConfigAddress::new(bus, device, function, 0x0c);
    ((read_data(addr) >> 16) & 0xff) as u8
}

fn read_bus_number(bus: u8, device: u8, function: u8) -> u32 {
    let addr = ConfigAddress::new(bus, device, function, 0x18);
    read_data(addr)
}

fn is_single_function_device(header_type: u8) -> bool {
    (header_type & 0x80) == 0
}

fn scan_function(devices: &mut Devices, bus: u8, device: u8, function: u8) -> Result<(), ()> {
    let vendor_id = read_vendor_id(bus, device, function);
    let class_code = read_class_code(bus, device, function);
    let header_type = read_header_type(bus, device, function);

    devices
        .try_push(Device {
            bus,
            device,
            function,
            vendor_id,
            class_code,
            header_type,
        })
        .map_err(|_| ())?;

    let base = ((class_code >> 24) & 0xff) as u8;
    let sub = ((class_code >> 16) & 0xff) as u8;

    if base == 0x06 && sub == 0x04 {
        // standard PCI-PCI bridge
        let bus_number = read_bus_number(bus, device, function);
        let secondary_bus = ((bus_number >> 8) & 0xff) as u8;
        scan_bus(devices, secondary_bus)?;
    }

    Ok(())
}

fn scan_device(devices: &mut Devices, bus: u8, device: u8) -> Result<(), ()> {
    scan_function(devices, bus, device, 0)?;

    if is_single_function_device(read_header_type(bus, device, 0)) {
        return Ok(());
    }

    for function in 1..8 {
        if read_vendor_id(bus, device, function) == 0xffff {
            continue;
        }
        scan_function(devices, bus, device, function)?;
    }

    Ok(())
}

fn scan_bus(devices: &mut Devices, bus: u8) -> Result<(), ()> {
    for device in 0..32 {
        if read_vendor_id(bus, device, 0) == 0xffff {
            continue;
        }

        scan_device(devices, bus, device)?;
    }

    Ok(())
}

pub fn scan_all_bus() -> Result<Devices, ()> {
    let mut devices = Devices::new();

    let header_type = read_header_type(0, 0, 0);
    if is_single_function_device(header_type) {
        scan_bus(&mut devices, 0)?;
        return Ok(devices);
    }

    for bus in 1..8 {
        if read_vendor_id(0, 0, bus) == 0xffff {
            continue;
        }

        scan_bus(&mut devices, bus)?;
    }

    Ok(devices)
}
