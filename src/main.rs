use windows::Devices::Bluetooth::Advertisement::*;
use windows::Storage::Streams::DataWriter;
use tokio::io::AsyncReadExt;
use uuid::Uuid;

#[tokio::main]
async fn main() -> windows::core::Result<()> {
    let manufacturer_data = BluetoothLEManufacturerData::new()?;

    // Company ID:
    manufacturer_data.SetCompanyId(0x004C)?;

    let data:[u8; 23] = [
        // Type:
        0x02,
        // Data Length:
        0x15,
        // Proximity UUID:
        0x9b, 0xCE, 0x37, 0xD2,
        0xAC, 0x85, 0x4E, 0x7E,
        0x83, 0x96, 0xF2, 0x23, 
        0x3A, 0xF7, 0x45, 0x81,
        // Major:
        0x00, 0x0A,
        // Minor:
        0x00, 0x28,
        // TX Power:
        0xC8
    ];

    let writer = DataWriter::new()?;
    writer.WriteBytes(&data)?;

    let buffer = writer.DetachBuffer()?;
    manufacturer_data.SetData(&buffer)?;

    let publisher = BluetoothLEAdvertisementPublisher::new()?;
    publisher.Advertisement()?.ManufacturerData()?.Append(&manufacturer_data)?;
    publisher.Start()?;

    print_data(&data);

    let mut stdin = tokio::io::stdin();
    println!("\nEnter `q` or `quit` to end the program.");
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Ok(n) => n,
            Err(_) => break
        };
        buf.truncate(n);
        let line = String::from_utf8(buf).expect("Found invalid UTF-8");
        let s = match line.strip_suffix("\r\n") {
            Some(s) => s,
            None => ""
        };
        if s == "q" || s == "quit" { break; }
    }

    publisher.Stop()?;
    println!("\nStopped.");
    Ok(())
}

fn print_data(data: &[u8]) {
    if data[0] == 0x02 && data[1] == 0x15 && data.len() == 23 {
        let uuid_string = Uuid::from_fields(
            u32::from_be_bytes([data[2], data[3], data[4], data[5]]),
            u16::from_be_bytes([data[6], data[7]]),
            u16::from_be_bytes([data[8], data[9]]),
            &data[10 .. 18].try_into().unwrap()
        ).hyphenated().to_string();

        let major = u16::from_be_bytes([data[18], data[19]]);
        let minor = u16::from_be_bytes([data[20], data[21]]);
        let tx_power = data[22] as i8;

        println!("- UUID: {}", uuid_string);
        println!("- Major: {}", major);
        println!("- Minor: {}", minor);
        println!("- TX Power: {}", tx_power);
    } else {
        println!("- Data does not conform to beacon format.");
    }
}
