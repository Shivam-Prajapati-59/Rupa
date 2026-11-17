// usage: capture <out.bin> <secs> <cols> <rows> <resize_storm 0|1> <cmd> [args...]
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = &a[1];
    let secs: u64 = a[2].parse().unwrap();
    let cols: u16 = a[3].parse().unwrap();
    let rows: u16 = a[4].parse().unwrap();
    let storm = a[5] == "1";
    let pty = native_pty_system();
    let pair = pty.openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }).unwrap();
    let mut cmd = CommandBuilder::new(&a[6]);
    for x in &a[7..] { cmd.arg(x); }
    cmd.env("TERM", "xterm-256color");
    let t0 = Instant::now();
    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);
    let mut reader = pair.master.try_clone_reader().unwrap();
    let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
    let b2 = buf.clone();
    let first = Arc::new(Mutex::new(None::<Duration>));
    let f2 = first.clone();
    std::thread::spawn(move || {
        let mut tmp = [0u8; 8192];
        loop {
            match reader.read(&mut tmp) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let mut f = f2.lock().unwrap();
                    if f.is_none() { *f = Some(t0.elapsed()); }
                    b2.lock().unwrap().extend_from_slice(&tmp[..n]);
                }
            }
        }
    });
    let mut writer = pair.master.take_writer().unwrap();
    let _ = writer.flush();
    let end = Instant::now() + Duration::from_secs(secs);
    let mut i = 0u32;
    while Instant::now() < end {
        std::thread::sleep(Duration::from_millis(10));
        if storm && i < 100 {
            let c = cols - (i % 20) as u16; let r = rows - (i % 8) as u16;
            let t = Instant::now();
            pair.master.resize(PtySize { rows: r, cols: c, pixel_width: 0, pixel_height: 0 }).unwrap();
            if t.elapsed() > Duration::from_millis(200) { println!("slow resize {} took {:?}", i, t.elapsed()); }
            i += 1;
        }
        if let Ok(Some(st)) = child.try_wait() { println!("child exited early: {:?}", st); break; }
    }
    if storm { pair.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }).unwrap(); std::thread::sleep(Duration::from_millis(500)); }
    let alive_before = child.try_wait().unwrap().is_none();
    println!("first output after {:?}; child alive at end: {}", first.lock().unwrap(), alive_before);
    let _ = child.kill();
    std::thread::sleep(Duration::from_millis(300));
    let data = buf.lock().unwrap().clone();
    std::fs::write(out, &data).unwrap();
    println!("captured {} bytes (resizes: {})", data.len(), i);
}
