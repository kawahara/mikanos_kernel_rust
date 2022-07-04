use arrayvec::ArrayVec;
use core::fmt::Formatter;
use core::ops::Range;
// To use set_bits and set_bit for numbers
use crate::log;
use crate::logger::Level as LogLevel;
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

impl Device {
    fn addr(&self, reg_addr: u8) -> ConfigAddress {
        ConfigAddress::new(self.bus, self.device, self.function, reg_addr)
    }

    pub fn read_bar(&self, bar_index: u8) -> Result<u64, ()> {
        if bar_index >= 6 {
            return Err(());
        }

        let addr = 0x10 + 4 * bar_index;
        let bar = read_data(self.addr(addr));

        // 32 bit address
        if (bar & 4) == 0 {
            return Ok(u64::from(bar));
        }

        // 64 bit address
        if bar_index >= 5 {
            return Err(());
        }

        let bar_upper = read_data(self.addr(addr + 4));
        Ok(u64::from(bar) | u64::from(bar_upper) << 32)
    }

    pub fn read_conf_reg(&self, reg_addr: u8) -> u32 {
        let addr = ConfigAddress::new(self.bus, self.device, self.function, reg_addr);
        read_data(addr)
    }

    pub fn write_conf_reg(&self, reg_addr: u8, value: u32) {
        let addr = ConfigAddress::new(self.bus, self.device, self.function, reg_addr);
        write_data(addr, value)
    }
}

impl core::fmt::Display for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02x}.{:02x}.{:02x} vend={:04x}, class={:08x}, head={:02x}",
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

fn write_data(addr: ConfigAddress, value: u32) {
    let mut ports = CONFIG.0.lock();
    unsafe {
        ports.addr.write(addr.0);
        ports.data.write(value)
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

pub fn switch_ehci_to_xhci(xhc_device: &Device, devices: &Devices) {
    let mut found_echi_device = false;
    for device in devices {
        // Find EHCI device
        if ((device.class_code >> 24) & 0xff) as u8 == 0x0c
            && ((device.class_code >> 16) & 0xff) as u8 == 0x03
            && ((device.class_code >> 8) & 0xff) as u8 == 0x20
        {
            found_echi_device = true;
        }
    }

    if !found_echi_device {
        // not found
        return;
    }

    // USB3PRM
    let supported_ports = xhc_device.read_conf_reg(0xdc);

    // USB3_PSSEN
    xhc_device.write_conf_reg(0xd8, supported_ports);

    // XUSB2PRM
    let ehci2xhci_ports = xhc_device.read_conf_reg(0xd4);

    // XUSB2PR
    xhc_device.write_conf_reg(0xd0, ehci2xhci_ports);
    log!(
        LogLevel::Debug,
        "SwitchEhci2Xhci: SS = {:2x}, xHCI = {:2x}\n",
        supported_ports,
        ehci2xhci_ports
    );
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsiTriggerMode {
    Edge,
    Level,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MsiDeliveryMode {
    Fixed = 0b000,
    LowestPriority = 0b001,
    Smi = 0b010,
    Nmi = 0b100,
    Init = 0b101,
    ExtInt = 0b111,
}

impl MsiDeliveryMode {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_u32(self) -> u32 {
        u32::from(self.as_u8())
    }
}

const CAPABILITY_MSI: u8 = 0x05;
const CAPABILITY_MSIX: u8 = 0x11;

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
struct CapabilityHeader {
    cap_id: u8,
    next_ptr: u8,
    cap: u16,
}

impl From<u32> for CapabilityHeader {
    fn from(value: u32) -> Self {
        let mut header = Self::default();
        unsafe { *(&mut header as *mut _ as *mut u32) = value };
        header
    }
}

impl CapabilityHeader {
    fn as_u32(self) -> u32 {
        unsafe { *(&self as *const _ as *const u32) }
    }

    const BIT_MSI_ENABLE: usize = 0;
    const BITS_MULTI_MSG_CAPABLE: Range<usize> = 1..4;
    const BITS_MULTI_MSG_ENABLE: Range<usize> = 4..7;
    const BIT_ADDR_64_CAPABLE: usize = 7;
    const BIT_PER_VECTOR_MASK_CAPABLE: usize = 8;

    fn set_msi_enable(&mut self, value: bool) -> &mut Self {
        let _ = self.cap.set_bit(Self::BIT_MSI_ENABLE, value);
        self
    }
    fn multi_msg_capable(self) -> u8 {
        self.cap.get_bits(Self::BITS_MULTI_MSG_CAPABLE) as u8
    }

    fn set_multi_msg_enable(&mut self, value: u8) -> &mut Self {
        let _ = self
            .cap
            .set_bits(Self::BITS_MULTI_MSG_ENABLE, u16::from(value));
        self
    }
    fn addr_64_capable(self) -> bool {
        self.cap.get_bit(Self::BIT_ADDR_64_CAPABLE)
    }
    fn per_vector_mask_capable(self) -> bool {
        self.cap.get_bit(Self::BIT_PER_VECTOR_MASK_CAPABLE)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct MsiCapability {
    header: CapabilityHeader,
    msg_addr: u32,
    msg_upper_addr: u32,
    msg_data: u32,
    mask_bits: u32,
    pending_bits: u32,
}

fn read_capability_header(device: &Device, cap_addr: u8) -> CapabilityHeader {
    CapabilityHeader::from(device.read_conf_reg(cap_addr))
}

fn read_msi_capability(device: &Device, cap_addr: u8) -> MsiCapability {
    let header = read_capability_header(device, cap_addr);
    let msg_addr = device.read_conf_reg(cap_addr + 4);
    let msg_upper_addr;

    let msg_data_addr;
    if header.addr_64_capable() {
        msg_upper_addr = device.read_conf_reg(cap_addr + 8);
        msg_data_addr = cap_addr + 12;
    } else {
        msg_upper_addr = 0;
        msg_data_addr = cap_addr + 8;
    }
    let msg_data = device.read_conf_reg(msg_data_addr);
    let mask_bits;
    let pending_bits;

    if header.per_vector_mask_capable() {
        mask_bits = device.read_conf_reg(msg_data_addr + 4);
        pending_bits = device.read_conf_reg(msg_data_addr + 8);
    } else {
        mask_bits = 0;
        pending_bits = 0;
    }

    MsiCapability {
        header,
        msg_addr,
        msg_upper_addr,
        msg_data,
        mask_bits,
        pending_bits,
    }
}

fn write_msi_capability(device: &Device, cap_addr: u8, msi_cap: MsiCapability) {
    device.write_conf_reg(cap_addr, msi_cap.header.as_u32());
    device.write_conf_reg(cap_addr + 4, msi_cap.msg_addr);

    let msg_data_addr;
    if msi_cap.header.addr_64_capable() {
        device.write_conf_reg(cap_addr + 8, msi_cap.msg_upper_addr);
        msg_data_addr = cap_addr + 12;
    } else {
        msg_data_addr = cap_addr + 8;
    }
    device.write_conf_reg(msg_data_addr, msi_cap.msg_data);

    if msi_cap.header.per_vector_mask_capable() {
        device.write_conf_reg(msg_data_addr + 4, msi_cap.mask_bits);
        device.write_conf_reg(msg_data_addr + 8, msi_cap.pending_bits);
    }
}

fn configure_msi_register(
    device: &Device,
    cap_addr: u8,
    msg_addr: u32,
    msg_data: u32,
    num_vector_exponent: u8,
) -> Result<(), ()> {
    let mut msi_cap = read_msi_capability(device, cap_addr);

    let multi_msg_enable = u8::min(msi_cap.header.multi_msg_capable(), num_vector_exponent);
    msi_cap.header.set_multi_msg_enable(multi_msg_enable);
    msi_cap.header.set_msi_enable(true);
    msi_cap.msg_addr = msg_addr;
    msi_cap.msg_data = msg_data;

    write_msi_capability(device, cap_addr, msi_cap);

    Ok(())
}

fn configure_msix_register(
    _device: &Device,
    _cap_addr: u8,
    _msg_addr: u32,
    _msg_data: u32,
    _num_vector_exponent: u8,
) -> Result<(), ()> {
    // Not Implemented
    Err(())
}

fn configure_msi(
    device: &Device,
    msg_addr: u32,
    msg_data: u32,
    num_vector_exponent: u8,
) -> Result<(), ()> {
    let mut cap_addr = (device.read_conf_reg(0x34) & 0xff) as u8;
    let mut msi_cap_addr = None;
    let mut msix_cap_addr = None;
    while cap_addr != 0 {
        let header = read_capability_header(device, cap_addr);
        match header.cap_id {
            CAPABILITY_MSI => msi_cap_addr = Some(cap_addr),
            CAPABILITY_MSIX => msix_cap_addr = Some(cap_addr),
            _ => {}
        }
        cap_addr = header.next_ptr;
    }
    if let Some(cap_addr) = msi_cap_addr {
        return configure_msi_register(device, cap_addr, msg_addr, msg_data, num_vector_exponent);
    }
    if let Some(cap_addr) = msix_cap_addr {
        return configure_msix_register(device, cap_addr, msg_addr, msg_data, num_vector_exponent);
    }
    Err(())
}

pub fn configure_msi_fixed_destination(
    device: &Device,
    apic_id: u32,
    trigger_mode: MsiTriggerMode,
    delivery_mode: MsiDeliveryMode,
    vector: u32,
    num_vector_exponent: u8,
) -> Result<(), ()> {
    let msg_addr = 0xfee00000 | (apic_id << 12);
    let mut msg_data = (delivery_mode.as_u32() << 8) | vector;
    if trigger_mode == MsiTriggerMode::Level {
        msg_data |= 0xc000;
    }
    configure_msi(device, msg_addr, msg_data, num_vector_exponent)?;
    Ok(())
}
