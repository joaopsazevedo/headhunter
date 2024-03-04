use headhunter::memory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cs2_process = unsafe { memory::get_cs2_process_id() }?;

    println!("CS2 pid: {}", cs2_process.pid);
    println!("CS2 base address: {}", cs2_process.base_address);
    Ok(())
}
