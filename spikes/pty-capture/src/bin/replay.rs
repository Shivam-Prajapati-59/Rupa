// usage: replay <in.bin> <cols> <rows>  -> prints vt100 screen + alacritty summary
use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::vte::ansi::Processor;
struct Sz(usize, usize);
impl Dimensions for Sz { fn total_lines(&self)->usize{self.1} fn screen_lines(&self)->usize{self.1} fn columns(&self)->usize{self.0} }
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let cols: u16 = a[2].parse().unwrap();
    let rows: u16 = a[3].parse().unwrap();
    let mut p = vt100::Parser::new(rows, cols, 1000);
    let t = std::time::Instant::now();
    p.process(&data);
    println!("vt100 processed {} bytes in {:?}", data.len(), t.elapsed());
    let s = p.screen();
    println!("alt screen: {} cursor: {:?} hidden: {}", s.alternate_screen(), s.cursor_position(), s.hide_cursor());
    println!("---- vt100 screen ----");
    for l in s.contents().lines() { println!("|{}", l); }
    // alacritty
    let mut term = Term::new(Config::default(), &Sz(cols as usize, rows as usize), VoidListener);
    let mut proc: Processor = Processor::new();
    let t = std::time::Instant::now();
    proc.advance(&mut term, &data);
    println!("alacritty processed in {:?}", t.elapsed());
    println!("---- alacritty screen ----");
    let g = term.grid();
    for line in 0..rows as i32 {
        let mut row = String::new();
        for col in 0..cols as usize {
            let c = g[alacritty_terminal::index::Line(line)][alacritty_terminal::index::Column(col)].c;
            row.push(c);
        }
        println!("|{}", row.trim_end());
    }
}
