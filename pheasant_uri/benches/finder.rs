use std::time::{Duration, Instant};

fn main() {
    let s = "http://127.0.0.1:9998/ftree?path=src&ssr&file=_File_1eed6_1&dir=_Dir_1eed6_13&chidren=_Children_1eed6_22&parent=_Parent_1eed6_20";
    let ch = 'h';

    // NOTE by_find is a few microseconds faster in debug mode
    // there is almost no different in release mode tho
    println!(
        "by_filter {:?} ---> {:?}",
        by_filter(s, ch),
        bench(s, ch, by_filter)
    );
    println!(
        "by_find   {:?} ---> {:?}",
        by_find(s, ch),
        bench(s, ch, by_find)
    );
}

fn by_filter(s: &str, ch: char) -> Vec<(usize, char)> {
    s.chars()
        .enumerate()
        .filter(|(_, c)| *c == ch)
        // .map(|(i, _)| i)
        .collect()
}

fn by_find(mut s: &str, ch: char) -> Vec<(usize, char)> {
    let mut v = vec![];
    let mut last = 0;

    while let Some(idx) = s.find(ch) {
        v.push((idx + last, ch));
        last += idx + 1;
        s = &s[idx + 1..];
    }

    v
}

fn bench(s: &str, ch: char, f: fn(s: &str, ch: char) -> Vec<(usize, char)>) -> Duration {
    let start = Instant::now();
    f(s, ch);

    Instant::now().duration_since(start)
}
