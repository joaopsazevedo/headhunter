use anyhow::anyhow;
use headhunter::memory;

fn get_cs2_process_id() -> Result<u32, anyhow::Error> {
    let mut process_ids = memory::get_process_ids("cs2.exe").into_iter();

    if let Some(process_id) = process_ids.next() {
        if process_ids.next().is_some() {
            Err(anyhow!(
                "Multiple CS2 processes found. Only one is allowed."
            ))
        } else {
            Ok(process_id)
        }
    } else {
        Err(anyhow!("No CS2 process was found."))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cs2_pid = get_cs2_process_id()?;

    println!("CS2 pid: {cs2_pid}");
    Ok(())
}
